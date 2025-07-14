use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;
use std::collections::HashMap;

use shared::*;
use shared::types::UpgradeType;
use crate::{AppState, game::{Game, TileType}};

pub async fn handle_client(socket: WebSocket, player_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Spawn task to forward messages from broadcast to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok((target_id, msg)) = rx.recv().await {
            // Send to all if target is nil, or to specific player
            if target_id == Uuid::nil() || target_id == player_id {
                let msg_json = serde_json::to_string(&msg).unwrap();
                if sender.send(Message::Text(msg_json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages from client
    let tx = state.tx.clone();
    let game = state.game.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                handle_client_message(player_id, client_msg, &game, &tx).await;
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Clean up player on disconnect
    {
        let mut game = state.game.write().await;
        game.remove_player(&player_id);
    }

    // Notify other players
    let _ = state.tx.send((Uuid::nil(), ServerMessage::PlayerDisconnected { player_id }));

    log::info!("Player {} disconnected", player_id);
}

async fn handle_client_message(
    player_id: Uuid,
    msg: ClientMessage,
    game: &tokio::sync::RwLock<Game>,
    tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    match msg {
        ClientMessage::JoinGame { player_name, preferred_team } => {
            let (team, spawn_pos) = {
                let mut game = game.write().await;
                game.add_player(player_id, player_name.clone(), preferred_team)
            };

            // Send join confirmation
            let join_msg = ServerMessage::JoinedGame {
                player_id,
                team,
                spawn_position: spawn_pos,
            };
            let _ = tx.send((player_id, join_msg));

            // Send full game state
            let state_msg = {
                let game = game.read().await;
                game.get_full_state()
            };
            let _ = tx.send((player_id, state_msg));

            log::info!("Player {} joined as {} on team {:?}", player_id, player_name, team);
        }

        ClientMessage::PlayerInput { direction, action_key_pressed } => {
            let mut game = game.write().await;
            
            // Handle movement
            if let Some(dir) = direction {
                handle_player_movement(&mut game, player_id, dir, tx).await;
            }

            // Handle action key
            if action_key_pressed {
                handle_action_key(&mut game, player_id, tx).await;
            }
        }

        ClientMessage::StationInput { button_index } => {
            let mut game = game.write().await;
            if let Some(player) = game.players.get(&player_id) {
                if let Some(station_id) = player.operating_station {
                    // Find the station and handle input
                    let station_info = {
                        let mut result = None;
                        for mech in game.mechs.values() {
                            if let Some(station) = mech.stations.get(&station_id) {
                                result = Some((mech.id, station.station_type));
                                break;
                            }
                        }
                        result
                    };

                    if let Some((mech_id, station_type)) = station_info {
                        handle_station_button(&mut game, mech_id, station_type, button_index, tx).await;
                    }
                }
            }
        }

        ClientMessage::ExitMech => {
            let mut game = game.write().await;
            handle_exit_mech(&mut game, player_id, tx).await;
        }

        ClientMessage::ChatMessage { message } => {
            if let Some(player) = game.read().await.players.get(&player_id) {
                let chat_msg = ServerMessage::ChatMessage {
                    player_id,
                    player_name: player.name.clone(),
                    message,
                    team_only: false,
                };
                let _ = tx.send((Uuid::nil(), chat_msg));
            }
        }
    }
}

async fn handle_player_movement(
    game: &mut Game,
    player_id: Uuid,
    dir: Direction,
    tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    if let Some(player) = game.players.get_mut(&player_id) {
        match player.location {
            PlayerLocation::OutsideWorld(pos) => {
                let (dx, dy) = dir.to_offset();
                let new_pos = pos.offset(dx, dy);
                
                // Check bounds
                if new_pos.x >= 0 && new_pos.x < ARENA_WIDTH_TILES &&
                   new_pos.y >= 0 && new_pos.y < ARENA_HEIGHT_TILES {
                    // Check tether distance if carrying resource
                    let mut can_move = true;
                    if player.carrying_resource.is_some() {
                        let player_team = player.team;
                        let min_dist = game.mechs.values()
                            .filter(|m| m.team == player_team)
                            .map(|m| new_pos.distance_to(&m.position))
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or(f32::MAX);
                        
                        if min_dist > MAX_DISTANCE_FROM_MECH {
                            can_move = false;
                        }
                    }

                    if can_move {
                        player.location = PlayerLocation::OutsideWorld(new_pos);
                        let _ = tx.send((Uuid::nil(), ServerMessage::PlayerMoved {
                            player_id,
                            location: player.location,
                        }));
                    }
                }
            }
            PlayerLocation::InsideMech { mech_id, mut floor, mut pos } => {
                // Get mech interior info
                let mech_info = game.mechs.get(&mech_id).map(|m| {
                    m.interior.floors.get(floor as usize).cloned()
                });

                if let Some(Some(floor_layout)) = mech_info {
                    let (dx, dy) = dir.to_offset();
                    let new_pos = pos.offset(dx, dy);
                    
                    // Check if new position is walkable
                    if new_pos.x >= 0 && new_pos.x < FLOOR_WIDTH_TILES &&
                       new_pos.y >= 0 && new_pos.y < FLOOR_HEIGHT_TILES {
                        let tile = floor_layout.tiles[new_pos.y as usize][new_pos.x as usize];
                        if tile != TileType::Wall && tile != TileType::Empty {
                            pos = new_pos;

                            // Check for ladder interaction
                            if tile == TileType::Ladder {
                                // Move up/down based on direction
                                match dir {
                                    Direction::Up if floor > 0 => floor -= 1,
                                    Direction::Down if floor < (MECH_FLOORS - 1) as u8 => floor += 1,
                                    _ => {}
                                }
                            }

                            let new_location = PlayerLocation::InsideMech { mech_id, floor, pos };
                            if let Some(player) = game.players.get_mut(&player_id) {
                                player.location = new_location;
                                let _ = tx.send((Uuid::nil(), ServerMessage::PlayerMoved {
                                    player_id,
                                    location: new_location,
                                }));
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn handle_action_key(
    game: &mut crate::game::Game,
    player_id: Uuid,
    tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    if let Some(player) = game.players.get(&player_id).cloned() {
        match player.location {
            PlayerLocation::OutsideWorld(pos) => {
                // Check for mech entry
                for mech in game.mechs.values() {
                    if mech.team == player.team {
                        let mech_entrance = mech.position.offset(-1, 0);
                        if pos.distance_to(&mech_entrance) < 2.0 {
                            // Enter mech
                            let new_location = PlayerLocation::InsideMech {
                                mech_id: mech.id,
                                floor: 0,
                                pos: TilePos::new(1, FLOOR_HEIGHT_TILES / 2),
                            };
                            if let Some(player) = game.players.get_mut(&player_id) {
                                player.location = new_location;
                                let _ = tx.send((Uuid::nil(), ServerMessage::PlayerMoved {
                                    player_id,
                                    location: new_location,
                                }));
                            }
                            return;
                        }
                    }
                }

                // Check for resource deposit
                if player.carrying_resource.is_some() {
                    for mech in game.mechs.values() {
                        if mech.team == player.team && pos.distance_to(&mech.position) < 5.0 {
                            // Deposit resource at mech
                            if let Some(player) = game.players.get_mut(&player_id) {
                                if let Some(resource_type) = player.carrying_resource.take() {
                                    let _ = tx.send((Uuid::nil(), ServerMessage::PlayerDroppedResource {
                                        player_id,
                                        resource_type,
                                        position: pos,
                                    }));
                                }
                            }
                            return;
                        }
                    }
                }
            }
            PlayerLocation::InsideMech { pos, floor, .. } => {
                // Check for station interaction
                let station_to_enter = game.mechs.values()
                    .flat_map(|m| m.stations.values())
                    .find(|s| s.floor == floor && s.position == pos && s.operated_by.is_none())
                    .map(|s| s.id);

                if let Some(station_id) = station_to_enter {
                    // Enter station
                    for mech in game.mechs.values_mut() {
                        if let Some(station) = mech.stations.get_mut(&station_id) {
                            station.operated_by = Some(player_id);
                            if let Some(player) = game.players.get_mut(&player_id) {
                                player.operating_station = Some(station_id);
                            }
                            let _ = tx.send((Uuid::nil(), ServerMessage::PlayerEnteredStation {
                                player_id,
                                station_id,
                            }));
                            return;
                        }
                    }
                } else if let Some(station_id) = player.operating_station {
                    // Exit station
                    for mech in game.mechs.values_mut() {
                        if let Some(station) = mech.stations.get_mut(&station_id) {
                            station.operated_by = None;
                        }
                    }
                    if let Some(player) = game.players.get_mut(&player_id) {
                        player.operating_station = None;
                    }
                    let _ = tx.send((Uuid::nil(), ServerMessage::PlayerExitedStation {
                        player_id,
                        station_id,
                    }));
                }
            }
        }
    }
}

async fn handle_exit_mech(
    game: &mut Game,
    player_id: Uuid,
    tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    if let Some(player) = game.players.get_mut(&player_id) {
        if let PlayerLocation::InsideMech { mech_id, .. } = player.location {
            // Exit station if operating one
            if let Some(station_id) = player.operating_station.take() {
                for mech in game.mechs.values_mut() {
                    if let Some(station) = mech.stations.get_mut(&station_id) {
                        station.operated_by = None;
                        let _ = tx.send((Uuid::nil(), ServerMessage::PlayerExitedStation {
                            player_id,
                            station_id,
                        }));
                        break;
                    }
                }
            }

            // Place player outside mech
            if let Some(mech) = game.mechs.get(&mech_id) {
                let exit_pos = mech.position.offset(-2, 0);
                player.location = PlayerLocation::OutsideWorld(exit_pos);
                let _ = tx.send((Uuid::nil(), ServerMessage::PlayerMoved {
                    player_id,
                    location: player.location,
                }));
            }
        }
    }
}

async fn handle_station_button(
    game: &mut Game,
    mech_id: Uuid,
    station_type: StationType,
    button_index: u8,
    tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    match station_type {
        StationType::WeaponLaser => {
            if button_index == 0 {
                // Fire laser - find nearest enemy mech
                let our_team = game.mechs.get(&mech_id).map(|m| m.team);
                if let Some(our_team) = our_team {
                    let target = game.mechs.values()
                        .filter(|m| m.team != our_team)
                        .min_by(|a, b| {
                            let our_pos = game.mechs.get(&mech_id).unwrap().position;
                            let dist_a = a.position.distance_to(&our_pos);
                            let dist_b = b.position.distance_to(&our_pos);
                            dist_a.partial_cmp(&dist_b).unwrap()
                        });

                    if let Some(target) = target {
                        let target_id = target.id;
                        let target_pos = target.position;
                        let target_health = target.health;
                        
                        let _ = tx.send((Uuid::nil(), ServerMessage::WeaponFired {
                            mech_id,
                            weapon_type: StationType::WeaponLaser,
                            target_position: target_pos,
                            projectile_id: None,
                        }));
                        
                        // Instant damage for laser
                        let damage = 10 * game.mechs.get(&mech_id).unwrap().upgrades.laser_level as u32;
                        let new_health = target_health.saturating_sub(damage);
                        
                        if let Some(target_mech) = game.mechs.get_mut(&target_id) {
                            target_mech.health = new_health;
                        }
                        
                        let _ = tx.send((Uuid::nil(), ServerMessage::MechDamaged {
                            mech_id: target_id,
                            damage,
                            health_remaining: new_health,
                        }));
                    }
                }
            }
        }
        StationType::WeaponProjectile => {
            if button_index == 0 {
                // Fire projectile
                let our_team = game.mechs.get(&mech_id).map(|m| m.team);
                if let Some(our_team) = our_team {
                    let target = game.mechs.values()
                        .filter(|m| m.team != our_team)
                        .min_by(|a, b| {
                            let our_pos = game.mechs.get(&mech_id).unwrap().position;
                            let dist_a = a.position.distance_to(&our_pos);
                            let dist_b = b.position.distance_to(&our_pos);
                            dist_a.partial_cmp(&dist_b).unwrap()
                        });

                    if let Some(target) = target {
                        let projectile_id = Uuid::new_v4();
                        let target_pos = target.position;
                        
                        // Calculate projectile trajectory
                        let start_pos = game.mechs.get(&mech_id).unwrap().position.to_world_pos();
                        let target_world = target_pos.to_world_pos();
                        let dx = target_world.x - start_pos.x;
                        let dy = target_world.y - start_pos.y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        let velocity = if dist > 0.0 {
                            (dx / dist * 300.0, dy / dist * 300.0)
                        } else {
                            (0.0, 0.0)
                        };

                        let projectile = crate::game::Projectile {
                            id: projectile_id,
                            position: start_pos,
                            velocity,
                            damage: 15 * game.mechs.get(&mech_id).unwrap().upgrades.projectile_level as u32,
                            owner_mech_id: mech_id,
                            lifetime: 5.0,
                        };
                        
                        game.projectiles.insert(projectile_id, projectile);

                        let _ = tx.send((Uuid::nil(), ServerMessage::WeaponFired {
                            mech_id,
                            weapon_type: StationType::WeaponProjectile,
                            target_position: target_pos,
                            projectile_id: Some(projectile_id),
                        }));
                    }
                }
            }
        }
        StationType::Shield => {
            if button_index == 0 {
                // Activate shield boost
                if let Some(mech) = game.mechs.get_mut(&mech_id) {
                    mech.shield = (mech.shield + 10).min(mech.max_shield);
                    let _ = tx.send((Uuid::nil(), ServerMessage::MechShieldChanged {
                        mech_id,
                        shield: mech.shield,
                    }));
                }
            }
        }
        StationType::Upgrade => {
            // Upgrade station - use resources to upgrade mech systems
            match button_index {
                0 => {
                    // Upgrade laser (costs 2 scrap metal + 1 computer component)
                    if check_and_consume_resources(game, mech_id, vec![
                        (ResourceType::ScrapMetal, 2),
                        (ResourceType::ComputerComponents, 1),
                    ]) {
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            mech.upgrades.laser_level = (mech.upgrades.laser_level + 1).min(5);
                            let _ = tx.send((Uuid::nil(), ServerMessage::MechUpgraded {
                                mech_id,
                                upgrade_type: UpgradeType::Laser,
                                new_level: mech.upgrades.laser_level,
                            }));
                        }
                    }
                }
                1 => {
                    // Upgrade projectile (costs 3 scrap metal)
                    if check_and_consume_resources(game, mech_id, vec![
                        (ResourceType::ScrapMetal, 3),
                    ]) {
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            mech.upgrades.projectile_level = (mech.upgrades.projectile_level + 1).min(5);
                            let _ = tx.send((Uuid::nil(), ServerMessage::MechUpgraded {
                                mech_id,
                                upgrade_type: UpgradeType::Projectile,
                                new_level: mech.upgrades.projectile_level,
                            }));
                        }
                    }
                }
                2 => {
                    // Upgrade shields (costs 2 batteries + 1 wiring)
                    if check_and_consume_resources(game, mech_id, vec![
                        (ResourceType::Batteries, 2),
                        (ResourceType::Wiring, 1),
                    ]) {
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            mech.upgrades.shield_level = (mech.upgrades.shield_level + 1).min(5);
                            mech.max_shield = 50 + (mech.upgrades.shield_level as u32 - 1) * 25;
                            let _ = tx.send((Uuid::nil(), ServerMessage::MechUpgraded {
                                mech_id,
                                upgrade_type: UpgradeType::Shield,
                                new_level: mech.upgrades.shield_level,
                            }));
                        }
                    }
                }
                3 => {
                    // Upgrade engine (costs 2 computer components + 2 wiring)
                    if check_and_consume_resources(game, mech_id, vec![
                        (ResourceType::ComputerComponents, 2),
                        (ResourceType::Wiring, 2),
                    ]) {
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            mech.upgrades.engine_level = (mech.upgrades.engine_level + 1).min(5);
                            let _ = tx.send((Uuid::nil(), ServerMessage::MechUpgraded {
                                mech_id,
                                upgrade_type: UpgradeType::Engine,
                                new_level: mech.upgrades.engine_level,
                            }));
                        }
                    }
                }
                _ => {}
            }
        }
        StationType::Repair => {
            if button_index == 0 {
                // Repair mech (costs 1 scrap metal per 20 HP)
                if let Some(mech) = game.mechs.get(&mech_id) {
                    let damage = mech.max_health.saturating_sub(mech.health);
                    let scrap_needed = (damage + 19) / 20; // Round up
                    
                    if scrap_needed > 0 && check_and_consume_resources(game, mech_id, vec![
                        (ResourceType::ScrapMetal, scrap_needed as usize),
                    ]) {
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            let healed = scrap_needed * 20;
                            mech.health = (mech.health + healed).min(mech.max_health);
                            let _ = tx.send((Uuid::nil(), ServerMessage::MechRepaired {
                                mech_id,
                                health_restored: healed,
                                new_health: mech.health,
                            }));
                        }
                    }
                }
            }
        }
        _ => {
            // Other stations not yet implemented
        }
    }
}

fn check_and_consume_resources(
    game: &mut Game,
    mech_id: Uuid,
    required: Vec<(ResourceType, usize)>,
) -> bool {
    // First check if mech has all required resources
    if let Some(mech) = game.mechs.get(&mech_id) {
        let mut available = HashMap::new();
        
        // Count resources deposited at this mech
        for resource in game.resources.values() {
            if resource.position.distance_to(&mech.position) < 5.0 {
                *available.entry(resource.resource_type).or_insert(0) += 1;
            }
        }
        
        // Check if we have enough of each type
        for (resource_type, needed) in &required {
            if available.get(resource_type).unwrap_or(&0) < needed {
                return false;
            }
        }
        
        // We have enough - consume the resources
        for (resource_type, needed) in required {
            let mut consumed = 0;
            let resources_to_remove: Vec<Uuid> = game.resources.iter()
                .filter(|(_, r)| {
                    if consumed >= needed {
                        return false;
                    }
                    if r.resource_type == resource_type && 
                       r.position.distance_to(&mech.position) < 5.0 {
                        consumed += 1;
                        true
                    } else {
                        false
                    }
                })
                .map(|(id, _)| *id)
                .collect();
                
            for id in resources_to_remove {
                game.resources.remove(&id);
            }
        }
        
        true
    } else {
        false
    }
}
use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;

use shared::*;
use shared::types::UpgradeType;
use crate::{AppState, game::Game};

pub async fn handle_client(socket: WebSocket, player_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Spawn task to forward messages from broadcast to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok((target_id, msg)) = rx.recv().await {
            // Send to all if target is nil, or to specific player
            if target_id == Uuid::nil() || target_id == player_id {
                let msg_json = match serde_json::to_string(&msg) {
                    Ok(json) => json,
                    Err(e) => {
                        log::error!("Failed to serialize message: {}", e);
                        break;
                    }
                };
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
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(client_msg) => {
                    // Validate the message before processing
                    if let Err(e) = client_msg.validate() {
                        log::warn!("Invalid message from player {}: {}", player_id, e);
                        // Optionally send error back to client
                        continue;
                    }
                    let command = crate::commands::create_command(client_msg);
                    if let Err(e) = command.execute(&game, player_id, &tx).await {
                        log::warn!("Command execution failed for player {}: {}", player_id, e);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to parse message from player {}: {}", player_id, e);
                }
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


pub async fn handle_player_movement(
    _game: &mut Game,
    _player_id: Uuid,
    _movement: (f32, f32),
    _tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    // Movement is now handled by the systems architecture (PhysicsSystem, TileBehaviorSystem)
    // This function is kept for compatibility but does nothing
}

pub async fn handle_action_key(
    game: &mut crate::game::Game,
    player_id: Uuid,
    tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    if let Some(player) = game.players.get(&player_id).cloned() {
        match player.location {
            PlayerLocation::OutsideWorld(pos) => {
                // Mech entry is now automatic by walking into the door, no action key needed

                // Check for resource deposit
                if player.carrying_resource.is_some() {
                    let player_tile = pos.to_tile_pos();
                    for mech in game.mechs.values_mut() {
                        if mech.team == player.team && player_tile.distance_to(mech.position) < MECH_COLLISION_DISTANCE {
                            // Deposit resource at mech
                            if let Some(player) = game.players.get_mut(&player_id) {
                                if let Some(resource_type) = player.carrying_resource.take() {
                                    // Add to mech inventory
                                    *mech.resource_inventory.entry(resource_type).or_insert(0) += 1;
                                    
                                    let _ = tx.send((Uuid::nil(), ServerMessage::PlayerDroppedResource {
                                        player_id,
                                        resource_type,
                                        position: player_tile,
                                    }));
                                }
                            }
                            return;
                        }
                    }
                }
            }
            PlayerLocation::InsideMech { pos, floor, .. } => {
                // First check if player is operating a station and wants to exit
                if let Some(station_id) = player.operating_station {
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
                    return; // Exit early - don't check for entering another station
                }
                
                // Otherwise check for station to enter
                let player_tile = pos.to_tile_pos();
                let station_to_enter = game.mechs.values()
                    .flat_map(|m| m.stations.values())
                    .find(|s| s.floor == floor && s.position == player_tile && s.operated_by.is_none())
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
                }
            }
        }
    }
}

pub async fn handle_exit_mech(
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
                let exit_tile = mech.position.offset(-2, 0);
                let exit_pos = WorldPos::new(exit_tile.x as f32 * TILE_SIZE, exit_tile.y as f32 * TILE_SIZE);
                player.location = PlayerLocation::OutsideWorld(exit_pos);
                let _ = tx.send((Uuid::nil(), ServerMessage::PlayerMoved {
                    player_id,
                    location: player.location,
                }));
            }
        }
    }
}

pub async fn handle_exit_station(
    game: &mut Game,
    player_id: Uuid,
    tx: &broadcast::Sender<(Uuid, ServerMessage)>,
) {
    if let Some(player) = game.players.get_mut(&player_id) {
        if let Some(station_id) = player.operating_station.take() {
            // Find and update the station
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
    }
}

pub async fn handle_station_button(
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
                let (our_team, our_pos, laser_level) = match game.mechs.get(&mech_id) {
                    Some(mech) => (mech.team, mech.position, mech.upgrades.laser_level),
                    None => {
                        log::error!("Mech {} not found when firing laser", mech_id);
                        return;
                    }
                };
                
                let target = game.mechs.values()
                    .filter(|m| m.team != our_team)
                    .min_by(|a, b| {
                        let dist_a = a.position.distance_to(our_pos);
                        let dist_b = b.position.distance_to(our_pos);
                        dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
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
                    let damage = LASER_BASE_DAMAGE + (LASER_DAMAGE_PER_LEVEL * (laser_level as u32 - 1));
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
        StationType::WeaponProjectile => {
            if button_index == 0 {
                // Fire projectile
                let (our_team, our_pos, projectile_level) = match game.mechs.get(&mech_id) {
                    Some(mech) => (mech.team, mech.position, mech.upgrades.projectile_level),
                    None => {
                        log::error!("Mech {} not found when firing projectile", mech_id);
                        return;
                    }
                };
                
                let target = game.mechs.values()
                    .filter(|m| m.team != our_team)
                    .min_by(|a, b| {
                        let dist_a = a.position.distance_to(our_pos);
                        let dist_b = b.position.distance_to(our_pos);
                        dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
                    });

                if let Some(target) = target {
                    let target_pos = target.position;
                    
                    // Calculate projectile trajectory
                    let start_pos = our_pos.to_world_pos();
                    let target_world = target_pos.to_world_pos();
                    let dx = target_world.x - start_pos.x;
                    let dy = target_world.y - start_pos.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let velocity = if dist > 0.0 {
                        (dx / dist * PROJECTILE_BASE_SPEED, dy / dist * PROJECTILE_BASE_SPEED)
                    } else {
                        (0.0, 0.0)
                    };

                    let damage = PROJECTILE_BASE_DAMAGE + (PROJECTILE_DAMAGE_PER_LEVEL * (projectile_level as u32 - 1));
                    
                    // Use the new pooled projectile system
                    let actual_projectile_id = game.create_projectile(
                        start_pos,
                        velocity,
                        damage,
                        mech_id,
                        PROJECTILE_LIFETIME,
                    );

                    let _ = tx.send((Uuid::nil(), ServerMessage::WeaponFired {
                        mech_id,
                        weapon_type: StationType::WeaponProjectile,
                        target_position: target_pos,
                        projectile_id: Some(actual_projectile_id),
                    }));
                }
            }
        }
        StationType::Shield => {
            if button_index == 0 {
                // Activate shield boost
                if let Some(mech) = game.mechs.get_mut(&mech_id) {
                    mech.shield = (mech.shield + SHIELD_BOOST_AMOUNT).min(mech.max_shield);
                    let _ = tx.send((Uuid::nil(), ServerMessage::MechShieldChanged {
                        mech_id,
                        shield: mech.shield,
                    }));
                }
            }
        }
        StationType::Engine => {
            // Engine station now uses WASD controls via EngineControl messages
            // Station buttons are not used for movement anymore
        }
        StationType::Upgrade => {
            // Upgrade station - use resources to upgrade mech systems
            match button_index {
                0 => {
                    // Upgrade laser (costs 2 scrap metal + 1 computer component)
                    if check_and_consume_resources(game, mech_id, upgrade_costs::LASER_UPGRADE.to_vec()) {
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            mech.upgrades.laser_level = (mech.upgrades.laser_level + 1).min(MAX_UPGRADE_LEVEL);
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
                    if check_and_consume_resources(game, mech_id, upgrade_costs::PROJECTILE_UPGRADE.to_vec()) {
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            mech.upgrades.projectile_level = (mech.upgrades.projectile_level + 1).min(MAX_UPGRADE_LEVEL);
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
                    if check_and_consume_resources(game, mech_id, upgrade_costs::SHIELD_UPGRADE.to_vec()) {
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            mech.upgrades.shield_level = (mech.upgrades.shield_level + 1).min(MAX_UPGRADE_LEVEL);
                            mech.max_shield = MECH_MAX_SHIELD + (mech.upgrades.shield_level as u32 - 1) * SHIELD_PER_LEVEL;
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
                    if check_and_consume_resources(game, mech_id, upgrade_costs::ENGINE_UPGRADE.to_vec()) {
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            mech.upgrades.engine_level = (mech.upgrades.engine_level + 1).min(MAX_UPGRADE_LEVEL);
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
                    let scrap_needed = (damage + REPAIR_HP_PER_SCRAP - 1) / REPAIR_HP_PER_SCRAP; // Round up
                    
                    if scrap_needed > 0 && check_and_consume_resources(game, mech_id, vec![
                        (ResourceType::ScrapMetal, scrap_needed as usize),
                    ]) {
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            let healed = scrap_needed * REPAIR_HP_PER_SCRAP;
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

pub async fn handle_engine_control(
    game: &mut Game,
    player_id: Uuid,
    movement: (f32, f32),
) {
    // Check if player is operating an engine station
    let player_station = game.players.get(&player_id)
        .and_then(|p| p.operating_station);
    
    if let Some(station_id) = player_station {
        // Find which mech contains this station
        let mech_to_control = game.mechs.values()
            .find(|m| m.stations.contains_key(&station_id))
            .map(|m| m.id);
        
        if let Some(mech_id) = mech_to_control {
            // Verify it's an engine station
            if let Some(mech) = game.mechs.get(&mech_id) {
                if let Some(station) = mech.stations.get(&station_id) {
                    if station.station_type == StationType::Engine {
                        // Update the specific mech's velocity based on WASD input
                        if let Some(mech) = game.mechs.get_mut(&mech_id) {
                            let base_speed = MECH_BASE_SPEED + (mech.upgrades.engine_level as f32 - 1.0) * MECH_SPEED_PER_LEVEL;
                            
                            // Normalize diagonal movement
                            let (mut vx, mut vy) = movement;
                            let magnitude = (vx * vx + vy * vy).sqrt();
                            if magnitude > 1.0 {
                                vx /= magnitude;
                                vy /= magnitude;
                            }
                            
                            // Apply speed to normalized velocity
                            mech.velocity = (vx * base_speed, vy * base_speed);
                        }
                    }
                }
            }
        }
    }
}

fn check_and_consume_resources(
    game: &mut Game,
    mech_id: Uuid,
    required: Vec<(ResourceType, usize)>,
) -> bool {
    // Check if mech has all required resources in its inventory
    if let Some(mech) = game.mechs.get(&mech_id) {
        // Check if we have enough of each type
        for (resource_type, needed) in &required {
            let available = mech.resource_inventory.get(resource_type).unwrap_or(&0);
            if available < &(*needed as u32) {
                return false;
            }
        }
        
        // We have enough - consume the resources from mech inventory
        if let Some(mech) = game.mechs.get_mut(&mech_id) {
            for (resource_type, needed) in required {
                if let Some(count) = mech.resource_inventory.get_mut(&resource_type) {
                    *count -= needed as u32;
                }
            }
        }
        
        true
    } else {
        false
    }
}
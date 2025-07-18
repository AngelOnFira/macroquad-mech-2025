use std::sync::{Arc, Mutex};
use std::thread;
use ws::{connect, Handler, Sender, Result, Message, CloseCode, Error};
use uuid::Uuid;

use shared::*;
use crate::game_state::GameState;

pub struct NetworkClient {
    sender: Sender,
}

impl NetworkClient {
    pub fn connect(url: &str, game_state: Arc<Mutex<GameState>>) -> Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        
        let url_clone = url.to_string();
        thread::spawn(move || {
            connect(url_clone, |out| {
                // Send the sender through the channel
                tx.send(out.clone()).unwrap();
                
                ClientHandler {
                    out,
                    game_state: Arc::clone(&game_state),
                }
            }).unwrap();
        });

        // Get the sender from the connection
        let sender = rx.recv().unwrap();
        
        Ok(NetworkClient { sender })
    }

    pub fn send_message(&self, msg: ClientMessage) {
        let json = serde_json::to_string(&msg).unwrap();
        self.sender.send(Message::Text(json)).unwrap();
    }
}

struct ClientHandler {
    out: Sender,
    game_state: Arc<Mutex<GameState>>,
}

impl Handler for ClientHandler {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        if let Message::Text(text) = msg {
            if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                self.handle_server_message(server_msg);
            }
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        log::info!("Connection closed: {:?} - {}", code, reason);
    }

    fn on_error(&mut self, err: Error) {
        log::error!("WebSocket error: {}", err);
    }
}

impl ClientHandler {
    fn handle_server_message(&mut self, msg: ServerMessage) {
        let mut game = self.game_state.lock().unwrap();

        match msg {
            ServerMessage::JoinedGame { player_id, team, spawn_position } => {
                game.player_id = Some(player_id);
                game.player_team = Some(team);
                game.player_location = PlayerLocation::OutsideWorld(spawn_position.to_world_pos());
                log::info!("Joined game as player {} on team {:?}", player_id, team);
            }

            ServerMessage::GameState { players, mechs, resources, projectiles } => {
                // Update full game state
                game.players.clear();
                for (id, player) in players {
                    game.players.insert(id, crate::game_state::PlayerData {
                        _id: player.id,
                        name: player.name,
                        team: player.team,
                        location: player.location,
                        carrying_resource: player.carrying_resource,
                    });
                }

                game.mechs.clear();
                for (id, mech) in mechs {
                    let mut mech_state = crate::game_state::MechState {
                        id: mech.id,
                        position: mech.position,
                        world_position: mech.world_position,
                        team: mech.team,
                        health: mech.health,
                        shield: mech.shield,
                        upgrades: mech.upgrades,
                        floors: vec![],
                        _resource_inventory: mech.resource_inventory,
                    };

                    // Build floor layouts
                    for floor_idx in 0..MECH_FLOORS {
                        mech_state.floors.push(crate::game_state::MechFloor::new(floor_idx as u8));
                    }

                    // Update stations
                    for station in mech.stations {
                        game.stations.insert(station.id, crate::game_state::StationState {
                            _id: station.id,
                            mech_id: mech.id,
                            floor: station.floor,
                            position: station.position,
                            station_type: station.station_type,
                            occupied: station.operated_by.is_some(),
                            operated_by: station.operated_by,
                        });
                    }

                    game.mechs.insert(id, mech_state);
                }

                game.resources.clear();
                for resource in resources {
                    game.resources.push(crate::game_state::ResourceState {
                        id: resource.id,
                        position: resource.position,
                        resource_type: resource.resource_type,
                    });
                }

                game.projectiles.clear();
                for proj in projectiles {
                    game.projectiles.push(crate::game_state::ProjectileData {
                        id: proj.id,
                        position: proj.position,
                        _velocity: proj.velocity,
                    });
                }
            }

            ServerMessage::PlayerMoved { player_id, location } => {
                if player_id == game.player_id.unwrap_or(Uuid::nil()) {
                    // Check if we're transitioning between outside and inside mech
                    let should_transition = match (&game.player_location, &location) {
                        (PlayerLocation::OutsideWorld(_), PlayerLocation::InsideMech { .. }) => {
                            Some(crate::game_state::TransitionType::EnteringMech)
                        }
                        (PlayerLocation::InsideMech { .. }, PlayerLocation::OutsideWorld(_)) => {
                            Some(crate::game_state::TransitionType::ExitingMech)
                        }
                        _ => None,
                    };

                    if let Some(transition_type) = should_transition {
                        game.transition = Some(crate::game_state::TransitionState {
                            _active: true,
                            transition_type,
                            progress: 0.0,
                            from_location: game.player_location,
                            to_location: location,
                        });
                    }

                    game.player_location = location;
                }
                if let Some(player) = game.players.get_mut(&player_id) {
                    player.location = location;
                }
            }

            ServerMessage::PlayerPickedUpResource { player_id, resource_type, resource_id } => {
                if let Some(player) = game.players.get_mut(&player_id) {
                    player.carrying_resource = Some(resource_type);
                }
                game.resources.retain(|r| r.id != resource_id);
            }

            ServerMessage::PlayerDroppedResource { player_id, resource_type, position } => {
                if let Some(player) = game.players.get_mut(&player_id) {
                    player.carrying_resource = None;
                }
                // Could add visual effect here
            }

            ServerMessage::PlayerEnteredStation { player_id, station_id } => {
                if player_id == game.player_id.unwrap_or(Uuid::nil()) {
                    // Check if it's a pilot station
                    let pilot_station_info = game.stations.get(&station_id)
                        .filter(|s| s.station_type == StationType::Pilot)
                        .map(|s| s.mech_id);
                    
                    if let Some(mech_id) = pilot_station_info {
                        // Open pilot window
                        game.ui_state.pilot_station_open = true;
                        game.ui_state.pilot_station_id = Some(station_id);
                        game.ui_state.operating_mech_id = Some(mech_id);
                    }
                }
                // Update station state
                if let Some(station) = game.stations.get_mut(&station_id) {
                    station.operated_by = Some(player_id);
                    station.occupied = true;
                }
            }

            ServerMessage::PlayerExitedStation { player_id, station_id } => {
                if player_id == game.player_id.unwrap_or(Uuid::nil()) {
                    // Close pilot window if it was open
                    if game.ui_state.pilot_station_id == Some(station_id) {
                        game.ui_state.pilot_station_open = false;
                        game.ui_state.pilot_station_id = None;
                        game.ui_state.operating_mech_id = None;
                    }
                }
                // Update station state
                if let Some(station) = game.stations.get_mut(&station_id) {
                    station.operated_by = None;
                    station.occupied = false;
                }
            }

            ServerMessage::MechMoved { mech_id, position, world_position } => {
                if let Some(mech) = game.mechs.get_mut(&mech_id) {
                    mech.position = position;
                    mech.world_position = world_position;
                }
            }

            ServerMessage::MechDamaged { mech_id, damage: _, health_remaining } => {
                if let Some(mech) = game.mechs.get_mut(&mech_id) {
                    mech.health = health_remaining;
                }
            }

            ServerMessage::MechShieldChanged { mech_id, shield } => {
                if let Some(mech) = game.mechs.get_mut(&mech_id) {
                    mech.shield = shield;
                }
            }

            ServerMessage::WeaponFired { mech_id, weapon_type, target_position, projectile_id } => {
                // Add visual effect
                game.weapon_effects.push(crate::game_state::WeaponEffect {
                    mech_id,
                    weapon_type,
                    target: target_position,
                    timer: WEAPON_EFFECT_DURATION,
                    _projectile_id: projectile_id,
                });
            }

            ServerMessage::ProjectileHit { projectile_id, .. } => {
                game.projectiles.retain(|p| p.id != projectile_id);
                // Could add explosion effect
            }

            ServerMessage::ResourceSpawned { resource_id, position, resource_type } => {
                game.resources.push(crate::game_state::ResourceState {
                    id: resource_id,
                    position,
                    resource_type,
                });
            }

            ServerMessage::PlayerDisconnected { player_id } => {
                game.players.remove(&player_id);
            }

            ServerMessage::MechUpgraded { mech_id, upgrade_type, new_level } => {
                if let Some(mech) = game.mechs.get_mut(&mech_id) {
                    match upgrade_type {
                        shared::UpgradeType::Laser => mech.upgrades.laser_level = new_level,
                        shared::UpgradeType::Projectile => mech.upgrades.projectile_level = new_level,
                        shared::UpgradeType::Shield => mech.upgrades.shield_level = new_level,
                        shared::UpgradeType::Engine => mech.upgrades.engine_level = new_level,
                    }
                }
                // Could add visual effect for upgrade completion
            }

            ServerMessage::MechRepaired { mech_id, health_restored: _, new_health } => {
                if let Some(mech) = game.mechs.get_mut(&mech_id) {
                    mech.health = new_health;
                }
                // Could add visual effect for repair
            }
            
            ServerMessage::PlayerKilled { player_id, killer: _, respawn_position } => {
                if player_id == game.player_id.unwrap_or(Uuid::nil()) {
                    // Player died - respawn them
                    game.player_location = PlayerLocation::OutsideWorld(respawn_position);
                }
                if let Some(player) = game.players.get_mut(&player_id) {
                    player.location = PlayerLocation::OutsideWorld(respawn_position);
                    player.carrying_resource = None;
                }
            }

            _ => {
                // Handle other messages as needed
            }
        }
    }
}
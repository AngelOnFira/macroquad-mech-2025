// WebSocket implementation for browsers using macroquad's JS interop
// This avoids wasm-bindgen to stay compatible with macroquad's loader

use std::sync::{Arc, Mutex};
use macroquad::prelude::*;
use shared::*;
use crate::game_state::GameState;

// JavaScript bindings for WebSocket using macroquad's sapp_jsutils
#[link(wasm_import_module = "network_bindings")]
extern "C" {
    fn js_ws_connect(url_ptr: *const u8, url_len: usize) -> u32;
    fn js_ws_send(socket_id: u32, data_ptr: *const u8, data_len: usize);
    fn js_ws_close(socket_id: u32);
    fn js_ws_is_connected(socket_id: u32) -> u32;
    fn js_ws_poll_message(socket_id: u32, buffer_ptr: *mut u8, buffer_len: usize) -> i32;
}

pub struct NetworkClient {
    socket_id: u32,
    game_state: Arc<Mutex<GameState>>,
    message_buffer: Vec<u8>,
}

impl NetworkClient {
    pub fn connect(url: &str, game_state: Arc<Mutex<GameState>>) -> Result<Self, String> {
        let socket_id = unsafe {
            js_ws_connect(url.as_ptr(), url.len())
        };
        
        if socket_id == 0 {
            return Err("Failed to create WebSocket".to_string());
        }
        
        info!("WebSocket connecting to: {}", url);
        
        Ok(NetworkClient {
            socket_id,
            game_state,
            message_buffer: vec![0u8; 65536], // 64KB buffer for messages
        })
    }
    
    pub fn update(&mut self) {
        // Poll for messages
        loop {
            let msg_len = unsafe {
                js_ws_poll_message(
                    self.socket_id,
                    self.message_buffer.as_mut_ptr(),
                    self.message_buffer.len()
                )
            };
            
            if msg_len < 0 {
                break; // No more messages
            }
            
            // Parse the message
            if let Ok(message_str) = std::str::from_utf8(&self.message_buffer[0..msg_len as usize]) {
                if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(message_str) {
                    handle_server_message(server_msg, &self.game_state);
                } else {
                    error!("Failed to parse server message: {}", message_str);
                }
            }
        }
    }
    
    pub fn send_message(&self, msg: ClientMessage) {
        if let Ok(json) = serde_json::to_string(&msg) {
            unsafe {
                js_ws_send(self.socket_id, json.as_ptr(), json.len());
            }
        }
    }
    
    pub fn is_connected(&self) -> bool {
        unsafe { js_ws_is_connected(self.socket_id) != 0 }
    }
}

impl Drop for NetworkClient {
    fn drop(&mut self) {
        unsafe {
            js_ws_close(self.socket_id);
        }
    }
}

fn handle_server_message(msg: ServerMessage, game_state: &Arc<Mutex<GameState>>) {
    let mut game = game_state.lock().unwrap();

    match msg {
        ServerMessage::JoinedGame { player_id, team, spawn_position } => {
            game.player_id = Some(player_id);
            game.player_team = Some(team);
            game.player_location = PlayerLocation::OutsideWorld(spawn_position.to_world_pos());
            info!("Joined game as player {} on team {:?}", player_id, team);
        }

        ServerMessage::GameState { players, mechs, resources, projectiles } => {
            // Update full game state
            game.players.clear();
            for (id, player) in players {
                game.players.insert(id, crate::game_state::PlayerData {
                    id: player.id,
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
                    resource_inventory: mech.resource_inventory.clone(),
                    floors: vec![],
                };

                // Build floor layouts
                for floor_idx in 0..MECH_FLOORS {
                    mech_state.floors.push(crate::game_state::MechFloor::new(floor_idx as u8));
                }

                // Update stations
                for station in &mech.stations {
                    game.stations.insert(station.id, crate::game_state::StationState {
                        id: station.id,
                        station_type: station.station_type,
                        mech_id: mech.id,
                        operated_by: station.operated_by,
                        floor: station.floor,
                        position: station.position,
                        occupied: station.operated_by.is_some(),
                    });
                }

                game.mechs.insert(id, mech_state);
            }

            // Update resources
            game.resources.clear();
            for resource in resources {
                game.resources.push(crate::game_state::ResourceState {
                    id: resource.id,
                    resource_type: resource.resource_type,
                    position: resource.position,
                });
            }

            // Update projectiles
            game.projectiles.clear();
            for projectile in projectiles {
                game.projectiles.push(crate::game_state::ProjectileData {
                    id: projectile.id,
                    position: projectile.position,
                    velocity: projectile.velocity,
                });
            }
        }

        ServerMessage::PlayerMoved { player_id, location } => {
            if let Some(player_data) = game.players.get_mut(&player_id) {
                player_data.location = location;
            }
        }

        ServerMessage::PlayerPickedUpResource { player_id, resource_id: _, resource_type } => {
            if let Some(player_data) = game.players.get_mut(&player_id) {
                player_data.carrying_resource = Some(resource_type);
            }
        }

        ServerMessage::MechMoved { mech_id, position, world_position } => {
            if let Some(mech_state) = game.mechs.get_mut(&mech_id) {
                mech_state.position = position;
                mech_state.world_position = world_position;
            }
        }

        ServerMessage::MechDamaged { mech_id, damage: _, health_remaining } => {
            if let Some(mech_state) = game.mechs.get_mut(&mech_id) {
                mech_state.health = health_remaining;
            }
        }

        ServerMessage::MechShieldChanged { mech_id, shield } => {
            if let Some(mech_state) = game.mechs.get_mut(&mech_id) {
                mech_state.shield = shield;
            }
        }

        ServerMessage::ResourceSpawned { resource_id, position, resource_type } => {
            game.resources.push(crate::game_state::ResourceState {
                id: resource_id,
                resource_type,
                position,
            });
        }

        ServerMessage::ResourceCollected { resource_id, player_id } => {
            game.resources.retain(|r| r.id != resource_id);
            if let Some(player) = game.players.get(&player_id) {
                info!("{} collected a resource", player.name);
            }
        }

        ServerMessage::WeaponFired { mech_id, weapon_type, target_position, projectile_id: _ } => {
            // Add visual effect
            if let Some(_mech) = game.mechs.get(&mech_id) {
                game.weapon_effects.push(crate::game_state::WeaponEffect {
                    mech_id,
                    target: target_position,
                    weapon_type,
                    timer: 0.0,
                    projectile_id: None,
                });
            }
        }

        ServerMessage::ProjectileHit { projectile_id, .. } => {
            game.projectiles.retain(|p| p.id != projectile_id);
        }

        ServerMessage::ProjectileExpired { projectile_id } => {
            game.projectiles.retain(|p| p.id != projectile_id);
        }

        ServerMessage::PlayerKilled { player_id, killer: _, respawn_position } => {
            if let Some(player_data) = game.players.get_mut(&player_id) {
                player_data.location = PlayerLocation::OutsideWorld(respawn_position);
            }
        }

        ServerMessage::Error { message } => {
            error!("Server error: {}", message);
        }

        _ => {
            // Ignore unhandled message types
        }
    }
}
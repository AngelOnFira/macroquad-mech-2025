use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent, ErrorEvent, CloseEvent, BinaryType};
use uuid::Uuid;
use shared::*;
use crate::game_state::GameState;

pub struct NetworkClient {
    socket: WebSocket,
    game_state: Arc<Mutex<GameState>>,
}

impl NetworkClient {
    pub fn connect(url: &str, game_state: Arc<Mutex<GameState>>) -> Result<Self, JsValue> {
        // Create WebSocket
        let socket = WebSocket::new(url)?;
        socket.set_binary_type(BinaryType::Arraybuffer);
        
        let game_state_clone = Arc::clone(&game_state);
        
        // Set up message handler
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let text: String = text.into();
                if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                    handle_server_message(server_msg, &game_state_clone);
                }
            }
        });
        socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
        
        // Set up error handler
        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
            web_sys::console::error_1(&format!("WebSocket error: {:?}", e).into());
        });
        socket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
        
        // Set up close handler
        let onclose_callback = Closure::<dyn FnMut(_)>::new(move |e: CloseEvent| {
            web_sys::console::log_1(&format!("WebSocket closed: code={}, reason={}", e.code(), e.reason()).into());
        });
        socket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();
        
        Ok(NetworkClient {
            socket,
            game_state,
        })
    }
    
    pub fn send_message(&self, msg: ClientMessage) {
        if self.socket.ready_state() == WebSocket::OPEN {
            if let Ok(json) = serde_json::to_string(&msg) {
                let _ = self.socket.send_with_str(&json);
            }
        }
    }
    
    pub fn is_connected(&self) -> bool {
        self.socket.ready_state() == WebSocket::OPEN
    }
    
    // Update method for WASM - no-op since WebSocket events are handled via callbacks
    pub fn update(&mut self) {
        // In WASM, WebSocket messages are handled asynchronously via callbacks
        // so there's nothing to poll here
    }
}

fn handle_server_message(msg: ServerMessage, game_state: &Arc<Mutex<GameState>>) {
    let mut game = game_state.lock().unwrap();

    match msg {
        ServerMessage::JoinedGame { player_id, team, spawn_position } => {
            game.player_id = Some(player_id);
            game.player_team = Some(team);
            game.player_location = PlayerLocation::OutsideWorld(spawn_position.to_world_pos());
            web_sys::console::log_1(&format!("Joined game as player {} on team {:?}", player_id, team).into());
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
                for station in mech.stations {
                    game.stations.insert(station.id, crate::game_state::StationState {
                        id: station.id,
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
                    velocity: proj.velocity,
                });
            }
        }

        ServerMessage::PlayerMoved { player_id, location } => {
            if player_id == game.player_id.unwrap_or(Uuid::nil()) {
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

        ServerMessage::PlayerDroppedResource { player_id, resource_type: _, position: _ } => {
            if let Some(player) = game.players.get_mut(&player_id) {
                player.carrying_resource = None;
            }
        }

        ServerMessage::PlayerEnteredStation { player_id, station_id } => {
            if let Some(station) = game.stations.get_mut(&station_id) {
                station.occupied = true;
                station.operated_by = Some(player_id);
            }
        }

        ServerMessage::PlayerExitedStation { player_id: _, station_id } => {
            if let Some(station) = game.stations.get_mut(&station_id) {
                station.occupied = false;
                station.operated_by = None;
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
                projectile_id,
            });
        }

        ServerMessage::ProjectileHit { projectile_id, .. } => {
            game.projectiles.retain(|p| p.id != projectile_id);
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
        }

        ServerMessage::MechRepaired { mech_id, health_restored: _, new_health } => {
            if let Some(mech) = game.mechs.get_mut(&mech_id) {
                mech.health = new_health;
            }
        }

        _ => {
            // Handle other messages as needed
        }
    }
}
use crate::game::Game;
use async_trait::async_trait;
use shared::*;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Command trait for handling client messages
#[async_trait]
pub trait Command: Send + Sync {
    async fn execute(
        &self,
        game: &tokio::sync::RwLock<Game>,
        player_id: Uuid,
        tx: &broadcast::Sender<(Uuid, ServerMessage)>,
    ) -> GameResult<()>;
}

/// Join game command
pub struct JoinGameCommand {
    pub player_name: String,
    pub preferred_team: Option<TeamId>,
}

#[async_trait]
impl Command for JoinGameCommand {
    async fn execute(
        &self,
        game: &tokio::sync::RwLock<Game>,
        player_id: Uuid,
        tx: &broadcast::Sender<(Uuid, ServerMessage)>,
    ) -> GameResult<()> {
        // Sanitize player name
        let sanitized_name = sanitize_player_name(&self.player_name);

        let (team, spawn_pos) = {
            let mut game = game.write().await;
            game.add_player(player_id, sanitized_name.clone(), self.preferred_team)
        };

        // Send join confirmation
        let join_msg = ServerMessage::JoinedGame {
            player_id,
            team,
            spawn_position: spawn_pos.to_tile(),
        };
        let _ = tx.send((player_id, join_msg));

        // Send full game state
        let state_msg = {
            let game = game.read().await;
            game.get_full_state()
        };
        let _ = tx.send((player_id, state_msg));
        
        // Send mech floor data immediately when player joins
        let floor_messages = {
            let game = game.read().await;
            game.get_mech_floor_data()
        };
        for floor_msg in floor_messages {
            let _ = tx.send((player_id, floor_msg));
        }

        log::info!("Player {player_id} joined as {sanitized_name} on team {team:?}");
        Ok(())
    }
}

/// Player input command
pub struct PlayerInputCommand {
    pub movement: (f32, f32),
    pub action_key_pressed: bool,
}

#[async_trait]
impl Command for PlayerInputCommand {
    async fn execute(
        &self,
        game: &tokio::sync::RwLock<Game>,
        player_id: Uuid,
        tx: &broadcast::Sender<(Uuid, ServerMessage)>,
    ) -> GameResult<()> {
        let mut game = game.write().await;

        // Handle movement
        if self.movement.0 != 0.0 || self.movement.1 != 0.0 {
            let movement_speed = shared::balance::PLAYER_MOVE_SPEED; // tiles per second
            let delta_time = shared::network_constants::FRAME_DELTA_SECONDS; // Use frame delta for consistent movement

            // Calculate movement delta
            let delta_x = self.movement.0 * movement_speed * TILE_SIZE * delta_time;
            let delta_y = self.movement.1 * movement_speed * TILE_SIZE * delta_time;

            // Collect tile event info before modifying player
            let mut tile_event_to_add: Option<shared::tile_entity::TileEvent> = None;

            // Get player location and calculate new position
            let (new_location, should_check_tile) = if let Some(player) =
                game.players.get(&player_id)
            {
                match &player.location {
                    PlayerLocation::OutsideWorld(pos) => {
                        // Check for collisions and calculate safe movement
                        let desired_movement = (delta_x, delta_y);
                        let safe_movement = {
                            // Create collision obstacles from all mechs
                            let mut obstacles = Vec::new();
                            for mech in game.mechs.values() {
                                obstacles.push(CollisionShape::mech(mech.world_position));
                            }
                            
                            let player_shape = CollisionShape::player(*pos);
                            CollisionUtils::calculate_safe_movement(
                                *pos,
                                desired_movement,
                                &player_shape,
                                &obstacles,
                            )
                        };

                        let mut new_pos = *pos;
                        new_pos.x += safe_movement.0;
                        new_pos.y += safe_movement.1;

                        // Keep within world bounds
                        new_pos.x = new_pos
                            .x
                            .max(0.0)
                            .min((ARENA_WIDTH_TILES as f32) * TILE_SIZE);
                        new_pos.y = new_pos
                            .y
                            .max(0.0)
                            .min((ARENA_HEIGHT_TILES as f32) * TILE_SIZE);

                        (PlayerLocation::OutsideWorld(new_pos), true)
                    }
                    PlayerLocation::InsideMech {
                        mech_id,
                        pos,
                    } => {
                        // Convert to local world position, apply movement, then convert back
                        let mut new_world_pos = pos.to_local_world();
                        new_world_pos.x += delta_x;
                        new_world_pos.y += delta_y;

                        // Keep within proper mech floor bounds
                        let floor_width_pixels = (shared::FLOOR_WIDTH_TILES as f32) * TILE_SIZE;
                        let floor_height_pixels = (shared::FLOOR_HEIGHT_TILES as f32) * TILE_SIZE;
                        new_world_pos.x = new_world_pos.x.max(0.0).min(floor_width_pixels);
                        new_world_pos.y = new_world_pos.y.max(0.0).min(floor_height_pixels);

                        // Convert back to MechInteriorPos, preserving floor
                        let new_pos = MechInteriorPos::new(pos.floor(), new_world_pos.to_tile());

                        (
                            PlayerLocation::InsideMech {
                                mech_id: *mech_id,
                                pos: new_pos,
                            },
                            false,
                        )
                    }
                }
            } else {
                return Ok(());
            };

            // Check for tile events at new position (only for OutsideWorld)
            if should_check_tile {
                if let PlayerLocation::OutsideWorld(pos) = new_location {
                    let tile_pos = pos.to_tile();
                    if let Some(tile_content) = game.tile_map.get_world_tile(tile_pos) {
                        if let shared::tile_entity::TileContent::Static(static_tile) = tile_content
                        {
                            if let Some(tile_event) = static_tile.on_enter(player_id) {
                                tile_event_to_add = Some(tile_event);
                            }
                        }
                    }
                }
            }

            // Update player position
            if let Some(player) = game.players.get_mut(&player_id) {
                player.location = new_location;

                // Send movement update to all players
                let _ = tx.send((
                    Uuid::nil(),
                    ServerMessage::PlayerMoved {
                        player_id,
                        location: player.location,
                    },
                ));
            }

            // Process tile event immediately to avoid timing issues
            if let Some(tile_event) = tile_event_to_add {
                match tile_event {
                    shared::tile_entity::TileEvent::BeginTransition {
                        actor,
                        zone_id: _,
                        transition_type,
                    } => {
                        match transition_type {
                            shared::tile_entity::TransitionType::MechEntrance { stage: _ } => {
                                // Process mech entry immediately
                                let mech_entry_info = if let Some(player) = game.players.get(&actor)
                                {
                                    if let PlayerLocation::OutsideWorld(pos) = player.location {
                                        let tile_pos = pos.to_tile();

                                        // Find the mech that owns this door tile
                                        let mut entry_info = None;
                                        for (mech_id, mech) in &game.mechs {
                                            let doors = shared::coordinates::MechDoorPositions::from_mech_position(mech.position);
                                            if tile_pos == doors.left_door
                                                || tile_pos == doors.right_door
                                            {
                                                // Check team access
                                                if mech.team == player.team {
                                                    let entry_world_pos =
                                                        doors.get_entry_position(tile_pos);
                                                    let entry_pos = MechInteriorPos::new(0, entry_world_pos.to_tile());
                                                    entry_info = Some((*mech_id, entry_pos));
                                                } else {
                                                    log::debug!(
                                                        "Player {actor} denied entry to enemy mech {mech_id}"
                                                    );
                                                }
                                                break;
                                            }
                                        }
                                        entry_info
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                // Update player location if entry is allowed
                                if let Some((mech_id, entry_pos)) = mech_entry_info {
                                    if let Some(player_mut) = game.players.get_mut(&actor) {
                                        player_mut.location = PlayerLocation::InsideMech {
                                            mech_id,
                                            pos: entry_pos,
                                        };

                                        // Send immediate update to all clients
                                        let _ = tx.send((
                                            Uuid::nil(),
                                            ServerMessage::PlayerMoved {
                                                player_id: actor,
                                                location: player_mut.location,
                                            },
                                        ));

                                        log::info!(
                                            "Player {actor} entered mech {mech_id} immediately"
                                        );
                                    }
                                }
                            }
                            _ => {
                                // For other tile events (like ladders), add to system queue for later processing
                                if let Some(tile_system) =
                                    game.system_manager
                                        .get_system_mut::<crate::systems::tile_behavior::TileBehaviorSystem>()
                                {
                                    tile_system.event_queue.push(shared::tile_entity::TileEvent::BeginTransition {
                                        actor,
                                        zone_id: 0,
                                        transition_type,
                                    });
                                }
                            }
                        }
                    }
                    _ => {
                        // For other tile events, add to system queue
                        if let Some(tile_system) = game
                            .system_manager
                            .get_system_mut::<crate::systems::tile_behavior::TileBehaviorSystem>(
                        ) {
                            tile_system.event_queue.push(tile_event);
                        }
                    }
                }
            }
        }

        // Handle action key
        if self.action_key_pressed {
            super::client::handle_action_key(&mut game, player_id, tx).await;
        }

        Ok(())
    }
}

/// Station input command
pub struct StationInputCommand {
    pub button_index: u8,
}

#[async_trait]
impl Command for StationInputCommand {
    async fn execute(
        &self,
        game: &tokio::sync::RwLock<Game>,
        player_id: Uuid,
        tx: &broadcast::Sender<(Uuid, ServerMessage)>,
    ) -> GameResult<()> {
        let mut game = game.write().await;

        let player = game
            .players
            .get(&player_id)
            .ok_or_else(|| GameError::player_not_found(player_id))?;

        let station_id = player
            .operating_station
            .ok_or(GameError::NotOperatingStation)?;

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
            super::client::handle_station_button(
                &mut game,
                mech_id,
                station_type,
                self.button_index,
                tx,
            )
            .await;
        } else {
            return Err(GameError::station_not_found(station_id));
        }

        Ok(())
    }
}

/// Engine control command
pub struct EngineControlCommand {
    pub movement: (f32, f32),
}

#[async_trait]
impl Command for EngineControlCommand {
    async fn execute(
        &self,
        game: &tokio::sync::RwLock<Game>,
        player_id: Uuid,
        _tx: &broadcast::Sender<(Uuid, ServerMessage)>,
    ) -> GameResult<()> {
        let mut game = game.write().await;
        super::client::handle_engine_control(&mut game, player_id, self.movement).await;
        Ok(())
    }
}

/// Exit mech command
pub struct ExitMechCommand;

#[async_trait]
impl Command for ExitMechCommand {
    async fn execute(
        &self,
        game: &tokio::sync::RwLock<Game>,
        player_id: Uuid,
        tx: &broadcast::Sender<(Uuid, ServerMessage)>,
    ) -> GameResult<()> {
        let mut game = game.write().await;
        super::client::handle_exit_mech(&mut game, player_id, tx).await;
        Ok(())
    }
}

/// Exit station command
pub struct ExitStationCommand;

#[async_trait]
impl Command for ExitStationCommand {
    async fn execute(
        &self,
        game: &tokio::sync::RwLock<Game>,
        player_id: Uuid,
        tx: &broadcast::Sender<(Uuid, ServerMessage)>,
    ) -> GameResult<()> {
        let mut game = game.write().await;
        super::client::handle_exit_station(&mut game, player_id, tx).await;
        Ok(())
    }
}

/// Chat message command
pub struct ChatMessageCommand {
    pub message: String,
}

#[async_trait]
impl Command for ChatMessageCommand {
    async fn execute(
        &self,
        game: &tokio::sync::RwLock<Game>,
        player_id: Uuid,
        tx: &broadcast::Sender<(Uuid, ServerMessage)>,
    ) -> GameResult<()> {
        let game = game.read().await;

        let player = game
            .players
            .get(&player_id)
            .ok_or_else(|| GameError::player_not_found(player_id))?;

        let chat_msg = ServerMessage::ChatMessage {
            player_id,
            player_name: player.name.clone(),
            message: self.message.clone(),
            team_only: false,
        };
        let _ = tx.send((Uuid::nil(), chat_msg));

        Ok(())
    }
}

/// Floor transition command for changing floors in mechs
pub struct FloorTransitionCommand {
    pub current_position: TilePos,
    pub target_floor: u8,
    pub stairway_position: TilePos,
}

#[async_trait]
impl Command for FloorTransitionCommand {
    async fn execute(
        &self,
        game: &tokio::sync::RwLock<Game>,
        player_id: Uuid,
        tx: &broadcast::Sender<(Uuid, ServerMessage)>,
    ) -> GameResult<()> {
        let mut game = game.write().await;

        let player = game
            .players
            .get(&player_id)
            .ok_or_else(|| GameError::player_not_found(player_id))?;

        // Check if player is in a mech
        if let PlayerLocation::InsideMech { mech_id, pos, .. } = player.location {
            let floor = pos.floor();
            // Validate target floor
            if self.target_floor >= 3 {
                let error_msg = ServerMessage::FloorTransitionFailed {
                    player_id,
                    reason: "Invalid floor number".to_string(),
                };
                let _ = tx.send((player_id, error_msg));
                return Ok(());
            }

            // Validate stairway position - check if there's actually a stairway there
            if let Some(mech_tilemap) = game.tile_map.mech_tiles.get(&mech_id) {
                if let Some(floor_map) = mech_tilemap.floors.get(floor as usize) {
                    if let Some(static_tile) = floor_map.static_tiles.get(&self.stairway_position) {
                        match static_tile {
                            StaticTile::TransitionZone { transition_type, .. } => {
                                let valid_transition = match transition_type {
                                    TransitionType::StairUp { target_floor, .. } => {
                                        *target_floor == self.target_floor
                                    }
                                    TransitionType::StairDown { target_floor, .. } => {
                                        *target_floor == self.target_floor
                                    }
                                    _ => false,
                                };

                                if !valid_transition {
                                    let error_msg = ServerMessage::FloorTransitionFailed {
                                        player_id,
                                        reason: "Invalid stairway target floor".to_string(),
                                    };
                                    let _ = tx.send((player_id, error_msg));
                                    return Ok(());
                                }
                            }
                            _ => {
                                let error_msg = ServerMessage::FloorTransitionFailed {
                                    player_id,
                                    reason: "No stairway at specified position".to_string(),
                                };
                                let _ = tx.send((player_id, error_msg));
                                return Ok(());
                            }
                        }
                    } else {
                        let error_msg = ServerMessage::FloorTransitionFailed {
                            player_id,
                            reason: "No tile at stairway position".to_string(),
                        };
                        let _ = tx.send((player_id, error_msg));
                        return Ok(());
                    }
                }
            }

            // Update player location
            if let Some(player) = game.players.get_mut(&player_id) {
                // Calculate new position on target floor - place near stairway
                let new_position = MechInteriorPos::new(self.target_floor, self.stairway_position);

                player.location = PlayerLocation::InsideMech {
                    mech_id,
                    pos: new_position,
                };

                // Update mech occupancy tracking
                if let Some(mech) = game.mechs.get_mut(&mech_id) {
                    mech.interior.set_player_floor(player_id, self.target_floor);
                }

                // Send success message
                let success_msg = ServerMessage::FloorTransitionComplete {
                    player_id,
                    mech_id,
                    old_floor: floor,
                    new_floor: self.target_floor,
                    new_position: self.stairway_position,
                };
                let _ = tx.send((Uuid::nil(), success_msg)); // Broadcast to all players

                Ok(())
            } else {
                Err(GameError::player_not_found(player_id))
            }
        } else {
            let error_msg = ServerMessage::FloorTransitionFailed {
                player_id,
                reason: "Player not in mech".to_string(),
            };
            let _ = tx.send((player_id, error_msg));
            Ok(())
        }
    }
}

/// Convert ClientMessage to Command
pub fn create_command(msg: ClientMessage) -> Box<dyn Command> {
    match msg {
        ClientMessage::JoinGame {
            player_name,
            preferred_team,
        } => Box::new(JoinGameCommand {
            player_name,
            preferred_team,
        }),
        ClientMessage::PlayerInput {
            movement,
            action_key_pressed,
        } => Box::new(PlayerInputCommand {
            movement,
            action_key_pressed,
        }),
        ClientMessage::StationInput { button_index } => {
            Box::new(StationInputCommand { button_index })
        }
        ClientMessage::EngineControl { movement } => Box::new(EngineControlCommand { movement }),
        ClientMessage::ExitMech => Box::new(ExitMechCommand),
        ClientMessage::ExitStation => Box::new(ExitStationCommand),
        ClientMessage::ChatMessage { message } => Box::new(ChatMessageCommand { message }),
        ClientMessage::FloorTransition { current_position, target_floor, stairway_position } => {
            Box::new(FloorTransitionCommand { 
                current_position, 
                target_floor, 
                stairway_position 
            })
        },
    }
}

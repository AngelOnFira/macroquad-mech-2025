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
            spawn_position: spawn_pos.to_tile_pos(),
        };
        let _ = tx.send((player_id, join_msg));

        // Send full game state
        let state_msg = {
            let game = game.read().await;
            game.get_full_state()
        };
        let _ = tx.send((player_id, state_msg));

        log::info!(
            "Player {player_id} joined as {sanitized_name} on team {team:?}"
        );
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
                        let mut new_pos = *pos;
                        // Move in world space
                        new_pos.x += delta_x;
                        new_pos.y += delta_y;

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
                        floor,
                        pos,
                    } => {
                        let mut new_pos = *pos;
                        // Move within mech interior bounds
                        new_pos.x += delta_x;
                        new_pos.y += delta_y;

                        // Keep within proper mech floor bounds
                        let floor_width_pixels = (shared::FLOOR_WIDTH_TILES as f32) * TILE_SIZE;
                        let floor_height_pixels = (shared::FLOOR_HEIGHT_TILES as f32) * TILE_SIZE;
                        new_pos.x = new_pos.x.max(0.0).min(floor_width_pixels);
                        new_pos.y = new_pos.y.max(0.0).min(floor_height_pixels);

                        (
                            PlayerLocation::InsideMech {
                                mech_id: *mech_id,
                                floor: *floor,
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
                    let tile_pos = pos.to_tile_pos();
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
                                        let tile_pos = pos.to_tile_pos();

                                        // Find the mech that owns this door tile
                                        let mut entry_info = None;
                                        for (mech_id, mech) in &game.mechs {
                                            let doors = shared::coordinates::MechDoorPositions::from_mech_position(mech.position);
                                            if tile_pos == doors.left_door
                                                || tile_pos == doors.right_door
                                            {
                                                // Check team access
                                                if mech.team == player.team {
                                                    let entry_pos =
                                                        doors.get_entry_position(tile_pos);
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
                                            floor: 0,
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
    }
}

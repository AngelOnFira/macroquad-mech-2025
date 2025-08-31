use async_trait::async_trait;
use tokio::sync::broadcast;
use uuid::Uuid;
use shared::*;
use crate::game::Game;

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

        log::info!("Player {} joined as {} on team {:?}", player_id, sanitized_name, team);
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
            // Update player position directly
            if let Some(player) = game.players.get_mut(&player_id) {
                let movement_speed = shared::balance::PLAYER_MOVE_SPEED; // tiles per second
                let delta_time = shared::network_constants::FRAME_DELTA_SECONDS; // Use frame delta for consistent movement
                
                // Calculate movement delta
                let delta_x = self.movement.0 * movement_speed * TILE_SIZE * delta_time;
                let delta_y = self.movement.1 * movement_speed * TILE_SIZE * delta_time;
                
                // Update position based on current location
                match &mut player.location {
                    PlayerLocation::OutsideWorld(pos) => {
                        // Move in world space
                        pos.x += delta_x;
                        pos.y += delta_y;
                        
                        // Keep within world bounds
                        pos.x = pos.x.max(0.0).min((ARENA_WIDTH_TILES as f32) * TILE_SIZE);
                        pos.y = pos.y.max(0.0).min((ARENA_HEIGHT_TILES as f32) * TILE_SIZE);
                    }
                    PlayerLocation::InsideMech { pos, .. } => {
                        // Move within mech interior bounds
                        pos.x += delta_x;
                        pos.y += delta_y;
                        
                        // Keep within mech interior bounds (simplified - could add proper collision)
                        let mech_bounds = (MECH_SIZE_TILES as f32) * TILE_SIZE;
                        pos.x = pos.x.max(0.0).min(mech_bounds);
                        pos.y = pos.y.max(0.0).min(mech_bounds);
                    }
                }
                
                // Send movement update to all players
                let _ = tx.send((Uuid::nil(), ServerMessage::PlayerMoved {
                    player_id,
                    location: player.location,
                }));
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
        
        let player = game.players.get(&player_id)
            .ok_or_else(|| GameError::player_not_found(player_id))?;
            
        let station_id = player.operating_station
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
            super::client::handle_station_button(&mut game, mech_id, station_type, self.button_index, tx).await;
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
        
        let player = game.players.get(&player_id)
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
        ClientMessage::JoinGame { player_name, preferred_team } => {
            Box::new(JoinGameCommand { player_name, preferred_team })
        }
        ClientMessage::PlayerInput { movement, action_key_pressed } => {
            Box::new(PlayerInputCommand { movement, action_key_pressed })
        }
        ClientMessage::StationInput { button_index } => {
            Box::new(StationInputCommand { button_index })
        }
        ClientMessage::EngineControl { movement } => {
            Box::new(EngineControlCommand { movement })
        }
        ClientMessage::ExitMech => {
            Box::new(ExitMechCommand)
        }
        ClientMessage::ExitStation => {
            Box::new(ExitStationCommand)
        }
        ClientMessage::ChatMessage { message } => {
            Box::new(ChatMessageCommand { message })
        }
    }
}
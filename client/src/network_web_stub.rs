use std::sync::{Arc, Mutex};
use uuid::Uuid;
use shared::*;
use crate::game_state::GameState;

pub struct NetworkClient {
    game_state: Arc<Mutex<GameState>>,
    url: String,
}

impl NetworkClient {
    pub fn connect(url: &str, game_state: Arc<Mutex<GameState>>) -> Result<Self, String> {
        log::warn!("WebSocket support not yet implemented for macroquad WASM build");
        log::info!("Game will run in offline mode");
        
        Ok(Self {
            game_state,
            url: url.to_string(),
        })
    }
    
    pub fn send_message(&self, _msg: ClientMessage) {
        // No-op for now
    }
    
    pub fn is_connected(&self) -> bool {
        false // Always disconnected in stub
    }
    
    pub fn update(&mut self) {
        // No-op for stub
    }
}
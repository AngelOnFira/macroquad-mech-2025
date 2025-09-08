use shared::ClientMessage;
use crate::game_state::GameState;
use std::sync::{Arc, Mutex};

pub trait NetworkClient {
    type Error;
    
    /// Connect to the server
    fn connect(url: &str, game_state: Arc<Mutex<GameState>>) -> Result<Self, Self::Error>
    where
        Self: Sized;
    
    /// Send a message to the server
    fn send_message(&self, msg: ClientMessage);
    
    /// Check if the connection is established
    fn is_connected(&self) -> bool;
}

/// Web-specific trait for polling-based updates
pub trait WebNetworkClient: NetworkClient {
    /// Update the connection and process messages
    /// This is needed for web implementations that use polling
    fn update(&mut self);
}
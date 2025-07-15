use std::sync::{Arc, Mutex};
use crate::{ClientMessage, ServerMessage, GameResult, NetworkError, NetworkResult};

/// Trait for network operations across platforms
pub trait NetworkTransport: Send + Sync {
    /// Send a client message to the server
    fn send_message(&self, msg: ClientMessage) -> NetworkResult<()>;
    
    /// Check if the connection is active
    fn is_connected(&self) -> bool;
    
    /// Close the connection
    fn close(&self);
    
    /// Get connection status information
    fn get_connection_info(&self) -> ConnectionInfo;
}

/// Information about the network connection
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub status: ConnectionStatus,
    pub url: String,
    pub retry_count: u32,
    pub last_error: Option<String>,
}

/// Current connection status
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error(String),
}

/// Network client that handles platform-specific transports
pub struct NetworkClient<T: NetworkTransport> {
    transport: T,
    connection_info: Arc<Mutex<ConnectionInfo>>,
    _retry_count: u32, // Will be used for retry logic in future
}

impl<T: NetworkTransport> NetworkClient<T> {
    pub fn new(transport: T, url: String) -> Self {
        let connection_info = ConnectionInfo {
            status: ConnectionStatus::Disconnected,
            url,
            retry_count: 0,
            last_error: None,
        };
        
        Self {
            transport,
            connection_info: Arc::new(Mutex::new(connection_info)),
            _retry_count: 0,
        }
    }
    
    pub fn send_message(&self, msg: ClientMessage) -> NetworkResult<()> {
        if !self.transport.is_connected() {
            return Err(NetworkError::ConnectionClosed);
        }
        
        self.transport.send_message(msg).map_err(|e| {
            self.update_connection_error(format!("Send failed: {}", e));
            e
        })
    }
    
    pub fn is_connected(&self) -> bool {
        self.transport.is_connected()
    }
    
    pub fn get_connection_info(&self) -> ConnectionInfo {
        self.connection_info.lock().unwrap().clone()
    }
    
    pub fn close(&self) {
        self.transport.close();
        self.update_connection_status(ConnectionStatus::Disconnected);
    }
    
    fn update_connection_status(&self, status: ConnectionStatus) {
        if let Ok(mut info) = self.connection_info.lock() {
            info.status = status;
        }
    }
    
    fn update_connection_error(&self, error: String) {
        if let Ok(mut info) = self.connection_info.lock() {
            info.status = ConnectionStatus::Error(error.clone());
            info.last_error = Some(error);
        }
    }
}

/// Helper trait for handling server messages
pub trait MessageHandler: Send + Sync {
    fn handle_server_message(&self, msg: ServerMessage) -> GameResult<()>;
}

/// Standard message handler that updates game state
pub struct GameStateHandler<T> {
    _game_state: Arc<Mutex<T>>, // Will be used for future message handling
}

impl<T> GameStateHandler<T> {
    pub fn new(game_state: Arc<Mutex<T>>) -> Self {
        Self { _game_state: game_state }
    }
}

/// Common message handling logic that can be shared across platforms
pub fn handle_server_message<T>(
    msg: ServerMessage,
    game_state: &Arc<Mutex<T>>,
    updater: impl Fn(&mut T, ServerMessage) -> GameResult<()>,
) -> GameResult<()> {
    match game_state.lock() {
        Ok(mut state) => updater(&mut state, msg),
        Err(_) => Err(crate::GameError::invalid_state("Failed to lock game state")),
    }
}

/// Serialize a client message to JSON
pub fn serialize_client_message(msg: &ClientMessage) -> NetworkResult<String> {
    serde_json::to_string(msg).map_err(NetworkError::from)
}

/// Deserialize a server message from JSON
pub fn deserialize_server_message(json: &str) -> NetworkResult<ServerMessage> {
    serde_json::from_str(json).map_err(NetworkError::from)
}

/// Auto-reconnect logic that can be shared across platforms
pub struct ReconnectManager {
    max_attempts: u32,
    current_attempts: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    last_attempt: Option<std::time::Instant>,
}

impl ReconnectManager {
    pub fn new(max_attempts: u32, base_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_attempts,
            current_attempts: 0,
            base_delay_ms,
            max_delay_ms,
            last_attempt: None,
        }
    }
    
    pub fn should_retry(&self) -> bool {
        self.current_attempts < self.max_attempts
    }
    
    pub fn can_attempt_now(&self) -> bool {
        match self.last_attempt {
            None => true,
            Some(last) => {
                let delay = self.calculate_delay();
                last.elapsed() >= std::time::Duration::from_millis(delay)
            }
        }
    }
    
    pub fn record_attempt(&mut self) {
        self.current_attempts += 1;
        self.last_attempt = Some(std::time::Instant::now());
    }
    
    pub fn reset(&mut self) {
        self.current_attempts = 0;
        self.last_attempt = None;
    }
    
    fn calculate_delay(&self) -> u64 {
        // Exponential backoff with jitter
        let base_delay = self.base_delay_ms * (2_u64.pow(self.current_attempts.min(10)));
        base_delay.min(self.max_delay_ms)
    }
    
    pub fn get_current_attempts(&self) -> u32 {
        self.current_attempts
    }
    
    pub fn get_max_attempts(&self) -> u32 {
        self.max_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnect_manager() {
        let mut manager = ReconnectManager::new(3, 100, 1000);
        
        assert!(manager.should_retry());
        assert!(manager.can_attempt_now());
        
        manager.record_attempt();
        assert_eq!(manager.get_current_attempts(), 1);
        assert!(!manager.can_attempt_now()); // Should wait for delay
        
        // Test reset
        manager.reset();
        assert_eq!(manager.get_current_attempts(), 0);
        assert!(manager.can_attempt_now());
    }
    
    #[test]
    fn test_message_serialization() {
        let msg = ClientMessage::ChatMessage {
            message: "Hello world".to_string(),
        };
        
        let json = serialize_client_message(&msg).unwrap();
        assert!(json.contains("ChatMessage"));
        assert!(json.contains("Hello world"));
    }
}
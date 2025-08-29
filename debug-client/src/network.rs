use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender as MpscSender, Receiver};
use ws::{connect, Handler, Sender as WsSender, Result as WsResult, Message, CloseCode, Handshake};
use serde_json;

use crate::{DebugMessage, DebugCommand};

/// WebSocket connection to the debug server
pub struct DebugConnection {
    tx: Arc<Mutex<Option<WsSender>>>,
    receiver: Arc<Mutex<Receiver<DebugMessage>>>,
    connected: Arc<Mutex<bool>>,
}

impl DebugConnection {
    /// Connect to the debug server
    pub fn connect(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (msg_tx, msg_rx) = channel();
        let connected = Arc::new(Mutex::new(false));
        let connected_clone = connected.clone();
        let tx = Arc::new(Mutex::new(None));
        let tx_clone = tx.clone();
        
        // Spawn WebSocket thread
        let url = url.to_string();
        std::thread::spawn(move || {
            if let Err(e) = connect(url, |out| {
                // Store the sender
                *tx_clone.lock().unwrap() = Some(out.clone());
                
                ClientHandler {
                    out,
                    tx: msg_tx.clone(),
                    connected: connected_clone.clone(),
                }
            }) {
                log::error!("Failed to connect: {}", e);
            }
        });
        
        // Wait a bit for connection
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        Ok(DebugConnection {
            tx,
            receiver: Arc::new(Mutex::new(msg_rx)),
            connected,
        })
    }
    
    /// Send a command to the server
    pub fn send_command(&self, cmd: DebugCommand) {
        if let Ok(tx_guard) = self.tx.lock() {
            if let Some(ref sender) = *tx_guard {
                if let Ok(json) = serde_json::to_string(&cmd) {
                    sender.send(Message::text(json)).ok();
                }
            }
        }
    }
    
    /// Poll for messages (non-blocking)
    pub fn poll_message(&self) -> Option<DebugMessage> {
        if let Ok(rx) = self.receiver.lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    }
    
    /// Check if still connected
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }
}

struct ClientHandler {
    out: WsSender,
    tx: MpscSender<DebugMessage>,
    connected: Arc<Mutex<bool>>,
}

impl Handler for ClientHandler {
    fn on_open(&mut self, _: Handshake) -> WsResult<()> {
        log::info!("Connected to debug server");
        *self.connected.lock().unwrap() = true;
        Ok(())
    }
    
    fn on_message(&mut self, msg: Message) -> WsResult<()> {
        if let Message::Text(text) = msg {
            // Try to parse as regular ServerMessage first
            if let Ok(server_msg) = serde_json::from_str::<shared::ServerMessage>(&text) {
                self.tx.send(DebugMessage::GameState(server_msg)).ok();
            } else if let Ok(debug_msg) = serde_json::from_str::<DebugMessage>(&text) {
                self.tx.send(debug_msg).ok();
            } else {
                log::warn!("Unknown message format: {}", text);
            }
        }
        Ok(())
    }
    
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        log::info!("Connection closed: {:?} - {}", code, reason);
        *self.connected.lock().unwrap() = false;
    }
    
    fn on_error(&mut self, err: ws::Error) {
        log::error!("WebSocket error: {}", err);
        *self.connected.lock().unwrap() = false;
    }
}
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(not(target_arch = "wasm32"))]
use ws::{connect, CloseCode, Error, Handler, Message, Result, Sender};

#[cfg(not(target_arch = "wasm32"))]
use crate::game_state::GameState;
#[cfg(not(target_arch = "wasm32"))]
use crate::network_common::handle_server_message;
#[cfg(not(target_arch = "wasm32"))]
use crate::network_trait::{NetworkClient as NetworkClientTrait};
#[cfg(not(target_arch = "wasm32"))]
use shared::*;

#[cfg(not(target_arch = "wasm32"))]
pub struct NetworkClient {
    sender: Sender,
}

#[cfg(not(target_arch = "wasm32"))]
impl NetworkClientTrait for NetworkClient {
    type Error = ws::Error;

    fn connect(url: &str, game_state: Arc<Mutex<GameState>>) -> std::result::Result<Self, Self::Error> {
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
            })
            .unwrap();
        });

        // Get the sender from the connection
        let sender = rx.recv().unwrap();

        Ok(NetworkClient { sender })
    }

    fn send_message(&self, msg: ClientMessage) {
        let json = serde_json::to_string(&msg).unwrap();
        self.sender.send(Message::Text(json)).unwrap();
    }

    fn is_connected(&self) -> bool {
        // For native ws, we assume connected if we have a sender
        // In a real implementation, we might want to track connection state
        true
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct ClientHandler {
    out: Sender,
    game_state: Arc<Mutex<GameState>>,
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
impl ClientHandler {
    fn handle_server_message(&mut self, msg: ServerMessage) {
        handle_server_message(msg, &self.game_state);
    }
}

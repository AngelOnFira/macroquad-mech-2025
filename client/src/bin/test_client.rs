use ws::{connect, Handler, Sender, Result, Message, CloseCode, Error};

use shared::*;

struct TestClient {
    out: Sender,
    player_name: String,
    player_id: Option<uuid::Uuid>,
}

impl Handler for TestClient {
    fn on_open(&mut self, _: ws::Handshake) -> Result<()> {
        println!("[{}] Connected to server", self.player_name);
        
        // Send join message
        let join_msg = ClientMessage::JoinGame {
            player_name: self.player_name.clone(),
            preferred_team: None,
        };
        
        let json = serde_json::to_string(&join_msg).unwrap();
        self.out.send(Message::Text(json))
    }
    
    fn on_message(&mut self, msg: Message) -> Result<()> {
        if let Message::Text(text) = msg {
            if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                match server_msg {
                    ServerMessage::JoinedGame { player_id, team, spawn_position } => {
                        self.player_id = Some(player_id);
                        println!("[{}] Joined game as player {} on team {:?} at {:?}", 
                            self.player_name, player_id, team, spawn_position);
                    }
                    ServerMessage::GameState { players, mechs, .. } => {
                        println!("[{}] Game state update:", self.player_name);
                        println!("  - {} players online", players.len());
                        println!("  - {} mechs in game", mechs.len());
                        
                        // Print other players
                        for (id, player) in &players {
                            if Some(*id) != self.player_id {
                                println!("  - Player {} ({}) on team {:?} at {:?}", 
                                    player.name, id, player.team, player.location);
                            }
                        }
                    }
                    ServerMessage::PlayerMoved { player_id, location } => {
                        if Some(player_id) != self.player_id {
                            println!("[{}] Player {} moved to {:?}", self.player_name, player_id, location);
                        }
                    }
                    ServerMessage::PlayerDisconnected { player_id } => {
                        println!("[{}] Player {} disconnected", self.player_name, player_id);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("[{}] Connection closed: {:?} - {}", self.player_name, code, reason);
    }
    
    fn on_error(&mut self, err: Error) {
        println!("[{}] Error: {}", self.player_name, err);
    }
}

fn main() {
    env_logger::init();
    
    // Get player name from args or use default
    let args: Vec<String> = std::env::args().collect();
    let player_name = args.get(1).cloned().unwrap_or_else(|| "TestPlayer".to_string());
    
    println!("Starting test client as '{}'...", player_name);
    
    let url = format!("ws://127.0.0.1:{}/ws", SERVER_PORT);
    
    // Connect and run
    if let Err(e) = connect(url, |out| {
        TestClient {
            out,
            player_name: player_name.clone(),
            player_id: None,
        }
    }) {
        println!("Failed to connect: {}", e);
    }
}
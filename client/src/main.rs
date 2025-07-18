use macroquad::prelude::*;
use std::sync::{Arc, Mutex};
use log::{info, error};

use shared::*;

mod game_state;
mod rendering;
mod input;

#[cfg(not(target_arch = "wasm32"))]
mod network;
#[cfg(target_arch = "wasm32")]
mod network_web_macroquad;

use game_state::GameState;
use rendering::Renderer;
use input::InputHandler;

#[cfg(not(target_arch = "wasm32"))]
use network::NetworkClient;
#[cfg(target_arch = "wasm32")]
use network_web_macroquad::NetworkClient;

#[macroquad::main("Mech Battle Arena")]
async fn main() {
    // Initialize logging
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    
    #[cfg(target_arch = "wasm32")]
    {
        // console_error_panic_hook::set_once();
        // For WASM, macroquad handles logging automatically
    }

    // Initialize game state
    let game_state = Arc::new(Mutex::new(GameState::new()));
    let renderer = Renderer::new();
    let mut input_handler = InputHandler::new();
    
    // Initialize network client
    let mut network_client: Option<NetworkClient>;
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::thread;
        
        let network_client_arc = Arc::new(Mutex::new(None));
        let net_clone = Arc::clone(&network_client_arc);
        let game_clone = Arc::clone(&game_state);
        
        // Connect to server in separate thread
        thread::spawn(move || {
            let server_url = format!("ws://127.0.0.1:{}/ws", SERVER_PORT);
            info!("Connecting to {}", server_url);
            
            match NetworkClient::connect(&server_url, game_clone) {
                Ok(client) => {
                    *net_clone.lock().unwrap() = Some(client);
                    info!("Connected to server!");
                }
                Err(e) => {
                    error!("Failed to connect: {}", e);
                }
            }
        });

        // Wait a bit for connection
        thread::sleep(std::time::Duration::from_millis(CONNECTION_RETRY_DELAY_MS));
        
        network_client = network_client_arc.lock().unwrap().take();
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        // For WASM, connect to localhost for development
        let server_url = format!("ws://127.0.0.1:{}/ws", SERVER_PORT);
        
        info!("Connecting to {}", server_url);
        
        network_client = match NetworkClient::connect(&server_url, Arc::clone(&game_state)) {
            Ok(client) => {
                info!("WebSocket created, waiting for connection...");
                Some(client)
            }
            Err(e) => {
                error!("Failed to create WebSocket: {:?}", e);
                None
            }
        };
    }

    // Wait for connection to establish
    let mut connection_wait = 0;
    while connection_wait < MAX_CONNECTION_ATTEMPTS { // Wait up to 2 seconds
        if let Some(ref client) = network_client {
            #[cfg(target_arch = "wasm32")]
            if client.is_connected() {
                break;
            }
            #[cfg(not(target_arch = "wasm32"))]
            break;
        }
        next_frame().await;
        connection_wait += 1;
    }

    // Send join request
    if let Some(ref client) = network_client {
        // Generate a random player name for demo
        let player_name = format!("Player{}", rand::gen_range(PLAYER_NAME_MIN_ID, PLAYER_NAME_MAX_ID));
        client.send_message(ClientMessage::JoinGame {
            player_name,
            preferred_team: None,
        });
    }

    loop {
        // Handle input
        let input = input_handler.update();
        
        // Send input to server
        if let Some(ref client) = network_client {
            // Check if we're operating a station
            let (operating_engine, operating_pilot) = {
                let game = game_state.lock().unwrap();
                if let Some(player_id) = game.player_id {
                    let operating_engine = game.stations.values().any(|station| {
                        station.operated_by == Some(player_id) && 
                        station.station_type == shared::types::StationType::Engine
                    });
                    let operating_pilot = game.stations.values().any(|station| {
                        station.operated_by == Some(player_id) && 
                        station.station_type == shared::types::StationType::Pilot
                    });
                    (operating_engine, operating_pilot)
                } else {
                    (false, false)
                }
            };
            
            if operating_engine || operating_pilot {
                // Send engine control for both engine and pilot stations
                if input.has_input() {
                    client.send_message(ClientMessage::EngineControl {
                        movement: input.movement,
                    });
                }
            } else {
                // Normal player movement
                if input.has_input() {
                    client.send_message(ClientMessage::PlayerInput {
                        movement: input.movement,
                        action_key_pressed: input.action_pressed,
                    });
                }
            }

            if input.exit_mech_pressed {
                client.send_message(ClientMessage::ExitMech);
            }

            // Handle station input (number keys 1-5)
            for i in 1..=5 {
                let key = match i {
                    1 => KeyCode::Key1,
                    2 => KeyCode::Key2,
                    3 => KeyCode::Key3,
                    4 => KeyCode::Key4,
                    5 => KeyCode::Key5,
                    _ => continue,
                };
                
                if is_key_pressed(key) {
                    client.send_message(ClientMessage::StationInput {
                        button_index: i - 1,
                    });
                }
            }
        }

        // Handle pilot window interactions
        {
            let mut game = game_state.lock().unwrap();
            
            // Handle ESC key to close pilot window
            if is_key_pressed(KeyCode::Escape) && game.ui_state.pilot_station_open {
                game.ui_state.pilot_station_open = false;
                game.ui_state.pilot_station_id = None;
                game.ui_state.operating_mech_id = None;
                
                // Exit station
                if let Some(ref client) = network_client {
                    client.send_message(ClientMessage::ExitStation);
                }
            }
            
            // Handle mouse clicks on pilot window
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mouse_x, mouse_y) = mouse_position();
                match rendering::is_pilot_window_clicked(&game, mouse_x, mouse_y) {
                    rendering::PilotWindowClick::Close => {
                        game.ui_state.pilot_station_open = false;
                        game.ui_state.pilot_station_id = None;
                        game.ui_state.operating_mech_id = None;
                        
                        // Exit station
                        if let Some(ref client) = network_client {
                            client.send_message(ClientMessage::ExitStation);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Update network client (for web)
        #[cfg(target_arch = "wasm32")]
        if let Some(ref mut client) = network_client {
            client.update();
        }
        
        // Update game state
        {
            let mut game = game_state.lock().unwrap();
            game.update(get_frame_time());
        }

        // Render
        clear_background(BLACK);
        {
            let game = game_state.lock().unwrap();
            renderer.render(&game);
        }
        
        // Draw connection status
        if network_client.is_none() {
            draw_text("Connecting to server...", CONNECTION_MESSAGE_X, CONNECTION_MESSAGE_Y, CONNECTION_STATUS_FONT_SIZE, WHITE);
        }

        next_frame().await
    }
}
use log::{error, info};
use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

use shared::*;

mod demo_mode;
mod game_state;
mod input;
mod rendering;
mod vision;

#[cfg(not(target_arch = "wasm32"))]
mod network;
#[cfg(target_arch = "wasm32")]
mod network_web_macroquad;

use game_state::GameState;
use input::InputHandler;
use rendering::Renderer;

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
        // For WASM, try to initialize basic logging
        info!("WASM client starting...");
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
    while connection_wait < MAX_CONNECTION_ATTEMPTS {
        // Wait up to 2 seconds
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
        let player_name = format!(
            "Player{}",
            rand::gen_range(PLAYER_NAME_MIN_ID, PLAYER_NAME_MAX_ID)
        );
        client.send_message(ClientMessage::JoinGame {
            player_name,
            preferred_team: None,
        });
    }

    // Profiling data
    struct FrameProfileData {
        input_time: f64,
        network_time: f64,
        game_update_time: f64,
        render_time: f64,
        total_frame_time: f64,
    }

    let mut frame_count = 0u32;
    let mut profile_history: Vec<FrameProfileData> = Vec::with_capacity(10);
    let mut last_profile_text = String::new();

    info!("Starting main game loop with profiling enabled");

    loop {
        let frame_start = get_time();

        // Check for demo mode
        if is_key_pressed(KeyCode::D) && is_key_down(KeyCode::LeftControl) {
            info!("Entering demo mode...");
            let mut demo = demo_mode::DemoMode::new();
            demo.run().await;
            info!("Exited demo mode");
        }

        // Handle input
        let input_start = get_time();
        let input = input_handler.update();
        let input_time = get_time() - input_start;

        // Send input to server
        let network_start = get_time();
        if let Some(ref client) = network_client {
            // Check if we're operating a station
            let (operating_engine, operating_pilot) = {
                let game = game_state.lock().unwrap();
                if let Some(player_id) = game.player_id {
                    let operating_engine = game.stations.values().any(|station| {
                        station.operated_by == Some(player_id)
                            && station.station_type == shared::types::StationType::Engine
                    });
                    let operating_pilot = game.stations.values().any(|station| {
                        station.operated_by == Some(player_id)
                            && station.station_type == shared::types::StationType::Pilot
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
        let network_time = get_time() - network_start;

        // Update game state
        let game_update_start = get_time();
        {
            let mut game = game_state.lock().unwrap();
            game.update(get_frame_time());
        }
        let game_update_time = get_time() - game_update_start;

        // Render
        let render_start = get_time();
        clear_background(BLACK);
        {
            let game = game_state.lock().unwrap();
            renderer.render(&game);
        }
        let render_time = get_time() - render_start;

        // Draw connection status
        if network_client.is_none() {
            draw_text(
                "Connecting to server...",
                CONNECTION_MESSAGE_X,
                CONNECTION_MESSAGE_Y,
                CONNECTION_STATUS_FONT_SIZE,
                WHITE,
            );
        }

        // Draw profiling info on screen
        if !last_profile_text.is_empty() {
            draw_text(&last_profile_text, 10.0, 300.0, 20.0, GREEN);
        }

        next_frame().await;

        // Profiling - collect frame data
        let total_frame_time = get_time() - frame_start;
        let frame_data = FrameProfileData {
            input_time,
            network_time,
            game_update_time,
            render_time,
            total_frame_time,
        };

        profile_history.push(frame_data);
        frame_count += 1;

        // Print profiling data every 10 frames
        if frame_count % 10 == 0 {
            let avg_input = profile_history.iter().map(|f| f.input_time).sum::<f64>()
                / profile_history.len() as f64;
            let avg_network = profile_history.iter().map(|f| f.network_time).sum::<f64>()
                / profile_history.len() as f64;
            let avg_game_update = profile_history
                .iter()
                .map(|f| f.game_update_time)
                .sum::<f64>()
                / profile_history.len() as f64;
            let avg_render = profile_history.iter().map(|f| f.render_time).sum::<f64>()
                / profile_history.len() as f64;
            let avg_total = profile_history
                .iter()
                .map(|f| f.total_frame_time)
                .sum::<f64>()
                / profile_history.len() as f64;

            let profile_msg = format!("Profiling: Input: {:.2}ms | Net: {:.2}ms | Update: {:.2}ms | Render: {:.2}ms | FPS: {:.0}", 
                avg_input * 1000.0,
                avg_network * 1000.0,
                avg_game_update * 1000.0,
                avg_render * 1000.0,
                1.0 / avg_total
            );

            last_profile_text = profile_msg.clone();

            info!("{}", profile_msg);

            // Also output to console for WASM debugging
            #[cfg(target_arch = "wasm32")]
            println!("{}", profile_msg);

            // Clear history for next batch
            profile_history.clear();
        }
    }
}

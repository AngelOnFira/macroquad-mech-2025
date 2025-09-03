use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

use shared::*;

mod demo_mode;
mod game_state;
mod input;
mod tracing_profiler;
mod rendering;
mod vision;
mod debug_overlay;

#[cfg(not(target_arch = "wasm32"))]
mod network;
#[cfg(target_arch = "wasm32")]
mod network_web_macroquad;

use game_state::GameState;
use input::InputHandler;
use tracing_profiler::TracingProfiler;
use rendering::{Renderer, RenderFlags};
use debug_overlay::DebugOverlay;

#[cfg(feature = "profiling")]
use profiling::scope;

#[cfg(feature = "profiling")]
use tracing_profiler::{info_span};

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
    let mut profiler = TracingProfiler::new();
    let mut debug_overlay = DebugOverlay::new();

    info!("Game state initialized");

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

    info!("Starting main game loop with profiling enabled");

    loop {
        #[cfg(feature = "profiling")]
        let _frame_span = info_span!("frame").entered();
        #[cfg(feature = "profiling")]
        scope!("frame");

        profiler.new_frame();
        profiler.handle_input();

        // Check for demo mode
        if is_key_pressed(KeyCode::D) && is_key_down(KeyCode::LeftControl) {
            info!("Entering demo mode...");
            let mut demo = demo_mode::DemoMode::new();
            demo.run().await;
            info!("Exited demo mode");
        }

        // Handle input
        let input = {
            #[cfg(feature = "profiling")]
            let _input_span = info_span!("input").entered();
            #[cfg(feature = "profiling")]
            scope!("input");
            input_handler.update()
        };

        // Send input to server
        {
            #[cfg(feature = "profiling")]
            let _network_span = info_span!("network").entered();
            #[cfg(feature = "profiling")]
            scope!("network");

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

        }

        // Update game state
        {
            #[cfg(feature = "profiling")]
            let _game_update_span = info_span!("game_update").entered();
            #[cfg(feature = "profiling")]
            scope!("game_update");

            let mut game = game_state.lock().unwrap();
            game.update(get_frame_time());
        }

        // Update debug overlay
        {
            let game = game_state.lock().unwrap();
            debug_overlay.update(&game, get_frame_time());
        }
        
        egui_macroquad::ui(|egui_ctx| {
            let game = game_state.lock().unwrap();
            debug_overlay.render_ui(egui_ctx, &game);
        });

        // Render
        {
            #[cfg(feature = "profiling")]
            let _render_span = info_span!("render").entered();
            #[cfg(feature = "profiling")]
            scope!("render");

            clear_background(BLACK);
            {
                let game = game_state.lock().unwrap();
                let render_flags = RenderFlags {
                    render_mechs: debug_overlay.render_mechs,
                    render_players: debug_overlay.render_players,
                    render_resources: debug_overlay.render_resources,
                    render_projectiles: debug_overlay.render_projectiles,
                    render_effects: debug_overlay.render_effects,
                    render_ui: debug_overlay.render_ui,
                    render_fog: debug_overlay.render_fog,
                    render_tiles: debug_overlay.render_tiles,
                    render_stations: debug_overlay.render_stations,
                };
                renderer.render_with_flags(&game, &render_flags);
            }
        }

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

        // Render profiler UI overlay
        profiler.render_ui();

        // Frame timing is automatically handled by RAII span guard

        // Log profiling stats to console
        profiler.log_frame_stats();

        egui_macroquad::draw();

        next_frame().await;
    }
}

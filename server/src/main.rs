use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade, Query, Path},
    response::IntoResponse,
    routing::{get, post},
    Router, Json,
};
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use tower::ServiceBuilder;
use uuid::Uuid;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use futures::{SinkExt, StreamExt};

use shared::*;

mod game;
mod client;
mod physics;
mod commands;
mod movement;
mod spatial_collision;
mod systems;

use game::Game;
use client::handle_client;

#[derive(Clone)]
pub struct AppState {
    pub game: Arc<RwLock<Game>>,
    pub tx: broadcast::Sender<(Uuid, ServerMessage)>,
}

#[derive(Debug, Deserialize)]
struct AddAIRequest {
    difficulty: Option<f32>,
    personality: Option<String>,
}

#[derive(Debug, Serialize)]
struct AddAIResponse {
    ai_id: Uuid,
    name: String,
    team: TeamId,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Create broadcast channel for game messages
    let (tx, _) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

    // Initialize game state
    let game = Arc::new(RwLock::new(Game::new()));

    // Initialize 2 mechs for testing
    {
        let mut game = game.write().await;
        game.create_initial_mechs();
        game.spawn_initial_resources();
    }

    let app_state = AppState {
        game: Arc::clone(&game),
        tx: tx.clone(),
    };

    // Start game update loop
    let game_loop = game.clone();
    let tx_loop = tx.clone();
    tokio::spawn(async move {
        game_loop::run_game_loop(game_loop, tx_loop).await;
    });

    // Build our application with routes
    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(websocket_handler))
        .route("/ai/add", post(add_ai_player))
        .route("/debug", get(debug_websocket_handler))
        .route("/debug/ai/:id", get(get_ai_debug_info))
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(cors_layer))
                .into_inner(),
        )
        .with_state(app_state);

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], SERVER_PORT));
    log::info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> &'static str {
    "Mech Battle Arena Server - Connect via WebSocket at /ws"
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let player_id = Uuid::new_v4();
    handle_client(socket, player_id, state).await;
}

async fn add_ai_player(
    State(state): State<AppState>,
    Json(request): Json<AddAIRequest>,
) -> Result<Json<AddAIResponse>, &'static str> {
    let difficulty = request.difficulty.unwrap_or(0.5).clamp(0.0, 1.0);
    
    // Parse personality
    let personality = request.personality.as_ref().and_then(|p| {
        match p.to_lowercase().as_str() {
            "aggressive" => Some(ai::Personality::Aggressive),
            "defensive" => Some(ai::Personality::Defensive),
            "support" => Some(ai::Personality::Support),
            "balanced" => Some(ai::Personality::Balanced),
            _ => None,
        }
    });
    
    // Add AI player to the game
    let mut game = state.game.write().await;
    
    if let Some(ai_id) = game.add_ai_player(difficulty, personality) {
        // Get player info for response
        if let Some(player) = game.players.get(&ai_id) {
            let response = AddAIResponse {
                ai_id,
                name: player.name.clone(),
                team: player.team,
            };
            
            // Broadcast game state update
            let game_state = game.get_full_state();
            let _ = state.tx.send((Uuid::nil(), game_state));
            
            Ok(Json(response))
        } else {
            Err("Failed to retrieve AI player info")
        }
    } else {
        Err("Failed to add AI player")
    }
}

// Simple CORS middleware
async fn cors_layer(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut response = next.run(req).await;
    response.headers_mut().insert(
        "Access-Control-Allow-Origin",
        match "*".parse() {
            Ok(val) => val,
            Err(e) => {
                log::error!("Failed to parse CORS origin header: {}", e);
                return response;
            }
        },
    );
    response.headers_mut().insert(
        "Access-Control-Allow-Methods",
        match "GET, POST, OPTIONS".parse() {
            Ok(val) => val,
            Err(e) => {
                log::error!("Failed to parse CORS methods header: {}", e);
                return response;
            }
        },
    );
    response.headers_mut().insert(
        "Access-Control-Allow-Headers",
        match "Content-Type, Upgrade, Connection".parse() {
            Ok(val) => val,
            Err(e) => {
                log::error!("Failed to parse CORS headers header: {}", e);
                return response;
            }
        },
    );
    response
}

mod game_loop {
    use super::*;
    use std::time::Duration;
    use tokio::time;

    pub async fn run_game_loop(
        game: Arc<RwLock<Game>>,
        tx: broadcast::Sender<(Uuid, ServerMessage)>,
    ) {
        let mut interval = time::interval(Duration::from_millis(FRAME_DURATION_MS)); // ~30 FPS

        loop {
            interval.tick().await;

            let mut game = game.write().await;
            
            // Update physics
            game.update_physics(FRAME_DELTA_SECONDS);

            // Check collisions
            game.check_resource_pickups(&tx);
            game.check_mech_entries(&tx);

            // Update projectiles
            game.update_projectiles(FRAME_DELTA_SECONDS, &tx);
            
            // Update mechs and check for player deaths
            let messages = game.update(FRAME_DELTA_SECONDS);
            for msg in messages {
                let _ = tx.send((Uuid::nil(), msg));
            }

            // Send periodic full state updates
            if game.tick_count % STATE_UPDATE_INTERVAL == 0 { // Every second
                let state_msg = game.get_full_state();
                let _ = tx.send((Uuid::nil(), state_msg));
            }

            game.tick_count += 1;
        }
    }
}
async fn debug_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_debug_socket(socket, state))
}

async fn handle_debug_socket(mut socket: WebSocket, state: AppState) {
    use axum::extract::ws::Message;
    use futures::{SinkExt, StreamExt};
    
    // For now, just send periodic game state updates
    let mut rx = state.tx.subscribe();
    
    let (mut sender, mut receiver) = socket.split();
    
    // Spawn task to handle incoming debug commands
    let game = state.game.clone();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Ok(text) = msg.to_text() {
                // Handle debug commands
                log::debug!("Debug command: {}", text);
            }
        }
    });
    
    // Send game updates to debug client
    while let Ok((_, msg)) = rx.recv().await {
        if let Ok(json) = serde_json::to_string(&msg) {
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    }
}

async fn get_ai_debug_info(
    Path(ai_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, &'static str> {
    let game = state.game.read().await;
    
    // Check if AI exists
    if !game.get_ai_players().contains(&ai_id) {
        return Err("AI not found");
    }
    
    Ok(Json(serde_json::json!({
        "ai_id": ai_id,
        "message": "Debug info would go here",
    })))
}

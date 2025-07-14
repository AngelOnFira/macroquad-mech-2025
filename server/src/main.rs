use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use tower::ServiceBuilder;
use uuid::Uuid;

use shared::*;

mod game;
mod client;
mod physics;

use game::Game;
use client::handle_client;

#[derive(Clone)]
pub struct AppState {
    pub game: Arc<RwLock<Game>>,
    pub tx: broadcast::Sender<(Uuid, ServerMessage)>,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // Create broadcast channel for game messages
    let (tx, _) = broadcast::channel(1000);

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
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(cors_layer))
                .into_inner(),
        )
        .with_state(app_state);

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], SERVER_PORT));
    log::info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
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

// Simple CORS middleware
async fn cors_layer(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut response = next.run(req).await;
    response.headers_mut().insert(
        "Access-Control-Allow-Origin",
        "*".parse().unwrap(),
    );
    response.headers_mut().insert(
        "Access-Control-Allow-Methods",
        "GET, POST, OPTIONS".parse().unwrap(),
    );
    response.headers_mut().insert(
        "Access-Control-Allow-Headers",
        "Content-Type, Upgrade, Connection".parse().unwrap(),
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
        let mut interval = time::interval(Duration::from_millis(33)); // ~30 FPS

        loop {
            interval.tick().await;

            let mut game = game.write().await;
            
            // Update physics
            game.update_physics(0.033);

            // Check collisions
            game.check_resource_pickups(&tx);
            game.check_mech_entries(&tx);

            // Update projectiles
            game.update_projectiles(0.033, &tx);

            // Send periodic full state updates
            if game.tick_count % 30 == 0 { // Every second
                let state_msg = game.get_full_state();
                let _ = tx.send((Uuid::nil(), state_msg));
            }

            game.tick_count += 1;
        }
    }
}
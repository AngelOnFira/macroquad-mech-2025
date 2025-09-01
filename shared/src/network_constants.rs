// Network and system constants

// ===== Network Configuration =====
pub const BROADCAST_CHANNEL_SIZE: usize = 1000;
pub const MESSAGE_BUFFER_SIZE: usize = 65536; // 64KB
pub const SERVER_ADDRESS: [u8; 4] = [127, 0, 0, 1];

// ===== Connection Settings =====
pub const MAX_CONNECTION_ATTEMPTS: u32 = 60; // frames to wait
pub const CONNECTION_RETRY_DELAY_MS: u64 = 100;

// ===== Game Loop Timing =====
pub const FRAME_DURATION_MS: u64 = 33; // ~30 FPS
pub const FRAME_DELTA_SECONDS: f32 = 0.033;
pub const STATE_UPDATE_INTERVAL: u64 = 30; // Send full state every second at 30 FPS

// ===== Player Configuration =====
pub const PLAYER_NAME_MIN_ID: u32 = 1000;
pub const PLAYER_NAME_MAX_ID: u32 = 9999;

// ===== Camera Settings =====
pub const DEFAULT_SPAWN_CAMERA_MULTIPLIER: f32 = 50.0;

// ===== Validation Limits =====
pub const MAX_PLAYER_NAME_LENGTH: usize = 32;
pub const MAX_CHAT_MESSAGE_LENGTH: usize = 256;
pub const MAX_MOVEMENT_MAGNITUDE: f32 = 2.0;
pub const MAX_STATION_BUTTONS: u8 = 8;

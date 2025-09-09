use crate::types::{ResourceType, StationType, TeamId};
use thiserror::Error;
use uuid::Uuid;

/// Main error type for the game
#[derive(Error, Debug)]
pub enum GameError {
    // Player-related errors
    #[error("Player {id} not found")]
    PlayerNotFound { id: Uuid },

    #[error("Player {id} is not in a valid location to perform this action")]
    InvalidPlayerLocation { id: Uuid },

    #[error("Player {id} is already carrying a resource")]
    AlreadyCarryingResource { id: Uuid },

    #[error("Player {id} is not carrying any resource")]
    NotCarryingResource { id: Uuid },

    #[error("Player {id} cannot perform action while operating station")]
    OperatingStation { id: Uuid },

    // Mech-related errors
    #[error("Mech {id} not found")]
    MechNotFound { id: Uuid },

    #[error("Mech at position ({x}, {y}) is full")]
    MechFull { x: i32, y: i32 },

    #[error("Cannot enter enemy mech (player team: {player_team:?}, mech team: {mech_team:?})")]
    WrongTeamMech {
        player_team: TeamId,
        mech_team: TeamId,
    },

    #[error("Mech {id} has insufficient health: {current}/{required}")]
    InsufficientMechHealth {
        id: Uuid,
        current: u32,
        required: u32,
    },

    // Station-related errors
    #[error("Station {id} not found")]
    StationNotFound { id: Uuid },

    #[error("Station {id} is already occupied by another player")]
    StationOccupied { id: Uuid },

    #[error("Station type {station_type:?} does not support this operation")]
    InvalidStationOperation { station_type: StationType },

    #[error("Player is not operating any station")]
    NotOperatingStation,

    // Resource-related errors
    #[error("Resource {id} not found")]
    ResourceNotFound { id: Uuid },

    #[error("No resource available for pickup at this location")]
    NoResourceAtLocation,

    #[error("Insufficient resources: need {required} {resource_type:?}, have {available}")]
    InsufficientResources {
        resource_type: ResourceType,
        required: u32,
        available: u32,
    },

    // Movement-related errors
    #[error("Invalid movement: position ({x}, {y}) is out of bounds")]
    OutOfBounds { x: f32, y: f32 },

    #[error("Invalid movement: collision detected at ({x}, {y})")]
    CollisionDetected { x: f32, y: f32 },

    #[error("Invalid floor transition: no ladder at current position")]
    NoLadderAtPosition,

    #[error("Cannot move to floor {floor}: invalid floor number")]
    InvalidFloor { floor: u8 },

    // Combat-related errors
    #[error("No valid target found for weapon")]
    NoValidTarget,

    #[error("Weapon on cooldown: {remaining_seconds:.1} seconds remaining")]
    WeaponOnCooldown { remaining_seconds: f32 },

    #[error("Invalid projectile {id}")]
    InvalidProjectile { id: Uuid },

    // Team-related errors
    #[error("Teams are unbalanced: cannot join {requested:?} team")]
    TeamsUnbalanced { requested: TeamId },

    #[error("Invalid team specified")]
    InvalidTeam,

    // Validation errors
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    #[error("Invalid game state: {message}")]
    InvalidGameState { message: String },
}

/// Network-specific errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Failed to serialize message (JSON): {0}")]
    JsonSerializationError(#[from] serde_json::Error),

    #[error("Failed to serialize message (MessagePack): {0}")]
    MessagePackEncodeError(#[from] rmp_serde::encode::Error),

    #[error("Failed to deserialize message (MessagePack): {0}")]
    MessagePackDecodeError(#[from] rmp_serde::decode::Error),

    #[error("Failed to send message: connection closed")]
    ConnectionClosed,

    #[error("Failed to receive message: {0}")]
    ReceiveError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("Client {id} is not authorized to perform this action")]
    Unauthorized { id: Uuid },
}

/// Input validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Movement vector is invalid: ({x}, {y})")]
    InvalidMovement { x: f32, y: f32 },

    #[error("Movement vector contains NaN or infinite values")]
    InvalidMovementNaN,

    #[error("Button index {index} is out of range (max: {max})")]
    InvalidButtonIndex { index: u8, max: u8 },

    #[error("Player name is invalid: {reason}")]
    InvalidPlayerName { reason: String },

    #[error("Message is too large: {size} bytes (max: {max})")]
    MessageTooLarge { size: usize, max: usize },

    #[error("Floor number is invalid (must be 0-2)")]
    InvalidFloorNumber,
}

/// Result type aliases for convenience
pub type GameResult<T> = Result<T, GameError>;
pub type NetworkResult<T> = Result<T, NetworkError>;
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Extension trait for adding context to errors
pub trait ErrorContext<T> {
    fn context(self, msg: &str) -> Result<T, GameError>;
}

impl<T> ErrorContext<T> for Option<T> {
    fn context(self, msg: &str) -> Result<T, GameError> {
        self.ok_or_else(|| GameError::InvalidGameState {
            message: msg.to_string(),
        })
    }
}

/// Helper functions for common error scenarios
impl GameError {
    pub fn player_not_found(id: Uuid) -> Self {
        GameError::PlayerNotFound { id }
    }

    pub fn mech_not_found(id: Uuid) -> Self {
        GameError::MechNotFound { id }
    }

    pub fn station_not_found(id: Uuid) -> Self {
        GameError::StationNotFound { id }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        GameError::InvalidInput {
            message: message.into(),
        }
    }

    pub fn invalid_state(message: impl Into<String>) -> Self {
        GameError::InvalidGameState {
            message: message.into(),
        }
    }
}

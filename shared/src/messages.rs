use crate::tile_entity::TileVisual;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Client -> Server Messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    JoinGame {
        player_name: String,
        preferred_team: Option<TeamId>,
    },
    PlayerInput {
        movement: (f32, f32), // normalized x, y velocity
        action_key_pressed: bool,
    },
    StationInput {
        button_index: u8,
    },
    EngineControl {
        movement: (f32, f32), // normalized x, y velocity for mech movement
    },
    ExitMech,
    ExitStation,
    ChatMessage {
        message: String,
    },
}

// Server -> Client Messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    // Connection
    JoinedGame {
        player_id: Uuid,
        team: TeamId,
        spawn_position: TilePos,
    },
    PlayerDisconnected {
        player_id: Uuid,
    },

    // Game State Updates
    GameState {
        players: HashMap<Uuid, PlayerState>,
        mechs: HashMap<Uuid, MechState>,
        resources: Vec<ResourceState>,
        projectiles: Vec<ProjectileState>,
    },

    // Player Updates
    PlayerMoved {
        player_id: Uuid,
        location: PlayerLocation,
    },
    PlayerPickedUpResource {
        player_id: Uuid,
        resource_type: ResourceType,
        resource_id: Uuid,
    },
    PlayerDroppedResource {
        player_id: Uuid,
        resource_type: ResourceType,
        position: TilePos,
    },
    PlayerEnteredStation {
        player_id: Uuid,
        station_id: Uuid,
    },
    PlayerExitedStation {
        player_id: Uuid,
        station_id: Uuid,
    },

    // Mech Updates
    MechMoved {
        mech_id: Uuid,
        position: TilePos,
        world_position: WorldPos,
    },
    MechDamaged {
        mech_id: Uuid,
        damage: u32,
        health_remaining: u32,
    },
    MechShieldChanged {
        mech_id: Uuid,
        shield: u32,
    },
    MechUpgraded {
        mech_id: Uuid,
        upgrade_type: UpgradeType,
        new_level: u8,
    },
    MechRepaired {
        mech_id: Uuid,
        health_restored: u32,
        new_health: u32,
    },

    // Combat
    WeaponFired {
        mech_id: Uuid,
        weapon_type: StationType,
        target_position: TilePos,
        projectile_id: Option<Uuid>,
    },
    ProjectileHit {
        projectile_id: Uuid,
        hit_mech_id: Option<Uuid>,
        damage_dealt: u32,
    },
    ProjectileExpired {
        projectile_id: Uuid,
    },
    EffectCreated {
        effect_id: Uuid,
        effect_type: String,
        position: WorldPos,
        duration: f32,
    },
    EffectExpired {
        effect_id: Uuid,
    },

    // Resources
    ResourceSpawned {
        resource_id: Uuid,
        position: TilePos,
        resource_type: ResourceType,
    },
    ResourceCollected {
        resource_id: Uuid,
        player_id: Uuid,
    },

    // Chat
    ChatMessage {
        player_id: Uuid,
        player_name: String,
        message: String,
        team_only: bool,
    },

    // Player death
    PlayerKilled {
        player_id: Uuid,
        killer: Option<Uuid>, // None if killed by environment (like being run over)
        respawn_position: WorldPos,
    },

    // Tile Updates
    TileUpdate {
        position: TilePos,
        visual: TileVisual,
    },
    TileBatch {
        tiles: Vec<(TilePos, TileVisual)>,
    },
    VisibilityUpdate {
        visible_tiles: Vec<(TilePos, TileVisual)>,
        player_position: WorldPos,
    },

    // Errors
    Error {
        message: String,
    },
}

// State structures for full game state sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: Uuid,
    pub name: String,
    pub team: TeamId,
    pub location: PlayerLocation,
    pub carrying_resource: Option<ResourceType>,
    pub operating_station: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechState {
    pub id: Uuid,
    pub team: TeamId,
    pub position: TilePos,
    pub world_position: WorldPos,
    pub health: u32,
    pub shield: u32,
    pub upgrades: MechUpgrades,
    pub stations: Vec<StationState>,
    pub resource_inventory: HashMap<ResourceType, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationState {
    pub id: Uuid,
    pub station_type: StationType,
    pub floor: u8,
    pub position: TilePos,
    pub operated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceState {
    pub id: Uuid,
    pub position: TilePos,
    pub resource_type: ResourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectileState {
    pub id: Uuid,
    pub position: WorldPos,
    pub velocity: (f32, f32),
    pub damage: u32,
    pub owner_mech_id: Uuid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MechUpgrades {
    pub laser_level: u8,
    pub projectile_level: u8,
    pub engine_level: u8,
    pub shield_level: u8,
}

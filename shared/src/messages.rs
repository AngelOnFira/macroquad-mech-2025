use crate::tile_entity::TileVisual;
use crate::mech_layout::{MechInterior, MechStation};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    FloorTransition {
        current_position: TilePos,
        target_floor: u8,
        stairway_position: TilePos,
    },
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
        player_id: PlayerId,
        team: TeamId,
        spawn_position: TilePos,
    },
    PlayerDisconnected {
        player_id: PlayerId,
    },

    // Game State Updates
    GameState {
        players: HashMap<PlayerId, PlayerState>,
        mechs: HashMap<MechId, MechState>,
        resources: Vec<ResourceState>,
        projectiles: Vec<ProjectileState>,
    },

    // Mech Floor Data - Complete floor layouts for clients
    MechFloorData {
        mech_id: MechId,
        interior: MechInterior,
        stations: HashMap<StationId, MechStation>,
    },

    // Floor transition success/failure
    FloorTransitionComplete {
        player_id: PlayerId,
        mech_id: MechId,
        old_floor: u8,
        new_floor: u8,
        new_position: TilePos,
    },
    FloorTransitionFailed {
        player_id: PlayerId,
        reason: String,
    },

    // Real-time mech interior updates (Future scope - not implemented in initial version)
    MechInteriorUpdate {
        mech_id: MechId,
        floor: u8,
        tile_updates: Vec<(TilePos, TileVisual)>,
        station_changes: Vec<StationUpdate>,
    },

    // Player Updates
    PlayerMoved {
        player_id: PlayerId,
        location: PlayerLocation,
    },
    PlayerPickedUpResource {
        player_id: PlayerId,
        resource_type: ResourceType,
        resource_id: ResourceId,
    },
    PlayerDroppedResource {
        player_id: PlayerId,
        resource_type: ResourceType,
        position: TilePos,
    },
    PlayerEnteredStation {
        player_id: PlayerId,
        station_id: StationId,
    },
    PlayerExitedStation {
        player_id: PlayerId,
        station_id: StationId,
    },

    // Mech Updates
    MechMoved {
        mech_id: MechId,
        position: TilePos,
        world_position: WorldPos,
    },
    MechDamaged {
        mech_id: MechId,
        damage: u32,
        health_remaining: u32,
    },
    MechShieldChanged {
        mech_id: MechId,
        shield: u32,
    },
    MechUpgraded {
        mech_id: MechId,
        upgrade_type: UpgradeType,
        new_level: u8,
    },
    MechRepaired {
        mech_id: MechId,
        health_restored: u32,
        new_health: u32,
    },

    // Combat
    WeaponFired {
        mech_id: MechId,
        weapon_type: StationType,
        target_position: TilePos,
        projectile_id: Option<ProjectileId>,
    },
    ProjectileHit {
        projectile_id: ProjectileId,
        hit_mech_id: Option<MechId>,
        damage_dealt: u32,
    },
    ProjectileExpired {
        projectile_id: ProjectileId,
    },
    EffectCreated {
        effect_id: WeaponEffectId,
        effect_type: String,
        position: WorldPos,
        duration: f32,
    },
    EffectExpired {
        effect_id: WeaponEffectId,
    },

    // Resources
    ResourceSpawned {
        resource_id: ResourceId,
        position: TilePos,
        resource_type: ResourceType,
    },
    ResourceCollected {
        resource_id: ResourceId,
        player_id: PlayerId,
    },

    // Chat
    ChatMessage {
        player_id: PlayerId,
        player_name: String,
        message: String,
        team_only: bool,
    },

    // Player death
    PlayerKilled {
        player_id: PlayerId,
        killer: Option<PlayerId>, // None if killed by environment (like being run over)
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

impl ServerMessage {
    pub fn type_name(&self) -> &'static str {
        match self {
            ServerMessage::JoinedGame { .. } => "JoinedGame",
            ServerMessage::PlayerDisconnected { .. } => "PlayerDisconnected",
            ServerMessage::GameState { .. } => "GameState",
            ServerMessage::MechFloorData { .. } => "MechFloorData",
            ServerMessage::FloorTransitionComplete { .. } => "FloorTransitionComplete",
            ServerMessage::FloorTransitionFailed { .. } => "FloorTransitionFailed",
            ServerMessage::MechInteriorUpdate { .. } => "MechInteriorUpdate",
            ServerMessage::PlayerMoved { .. } => "PlayerMoved",
            ServerMessage::PlayerPickedUpResource { .. } => "PlayerPickedUpResource",
            ServerMessage::PlayerDroppedResource { .. } => "PlayerDroppedResource",
            ServerMessage::PlayerEnteredStation { .. } => "PlayerEnteredStation",
            ServerMessage::PlayerExitedStation { .. } => "PlayerExitedStation",
            ServerMessage::MechMoved { .. } => "MechMoved",
            ServerMessage::MechDamaged { .. } => "MechDamaged",
            ServerMessage::MechShieldChanged { .. } => "MechShieldChanged",
            ServerMessage::MechUpgraded { .. } => "MechUpgraded",
            ServerMessage::MechRepaired { .. } => "MechRepaired",
                ServerMessage::WeaponFired { .. } => "WeaponFired",
            ServerMessage::ProjectileHit { .. } => "ProjectileHit",
            ServerMessage::ProjectileExpired { .. } => "ProjectileExpired",
            ServerMessage::EffectCreated { .. } => "EffectCreated",
            ServerMessage::EffectExpired { .. } => "EffectExpired",
            ServerMessage::ResourceSpawned { .. } => "ResourceSpawned",
            ServerMessage::ResourceCollected { .. } => "ResourceCollected",
            ServerMessage::ChatMessage { .. } => "ChatMessage",
            ServerMessage::PlayerKilled { .. } => "PlayerKilled",
            ServerMessage::TileUpdate { .. } => "TileUpdate",
            ServerMessage::TileBatch { .. } => "TileBatch",
            ServerMessage::VisibilityUpdate { .. } => "VisibilityUpdate",
            ServerMessage::Error { .. } => "Error",
        }
    }
}

// State structures for full game state sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: PlayerId,
    pub name: String,
    pub team: TeamId,
    pub location: PlayerLocation,
    pub carrying_resource: Option<ResourceType>,
    pub operating_station: Option<StationId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechState {
    pub id: MechId,
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
    pub id: StationId,
    pub station_type: StationType,
    pub floor: u8,
    pub position: TilePos,
    pub size: crate::mech_layout::StationSize, // Add multi-tile station support
    pub operated_by: Option<PlayerId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceState {
    pub id: ResourceId,
    pub position: TilePos,
    pub resource_type: ResourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectileState {
    pub id: ProjectileId,
    pub position: WorldPos,
    pub velocity: (f32, f32),
    pub damage: u32,
    pub owner_mech_id: MechId,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MechUpgrades {
    pub laser_level: u8,
    pub projectile_level: u8,
    pub engine_level: u8,
    pub shield_level: u8,
}

// Station update for real-time interior changes (Future scope)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StationUpdate {
    Damaged { station_id: StationId, damage_amount: u32 },
    Repaired { station_id: StationId, repair_amount: u32 },
    Upgraded { station_id: StationId, new_level: u8 },
    StatusChanged { station_id: StationId, new_status: String },
}

use crate::{floor_manager::FloorManager, vision::ClientVisionSystem};
use macroquad::prelude::*;
use shared::{constants::*, network_constants::*, tile_entity::TileVisual, types::*};
use std::collections::HashMap;
use uuid::Uuid;

pub struct GameState {
    pub player_id: Option<Uuid>,
    pub player_location: PlayerLocation,
    pub player_team: Option<TeamId>,
    pub players: HashMap<Uuid, PlayerData>,
    pub mechs: HashMap<Uuid, MechState>,
    pub stations: HashMap<Uuid, StationState>,
    pub resources: Vec<ResourceState>,
    pub projectiles: Vec<ProjectileData>,
    pub weapon_effects: Vec<WeaponEffect>,
    pub camera_offset: (f32, f32),
    pub ui_state: UIState,
    pub visible_tiles: HashMap<TilePos, TileVisual>,
    pub vision_system: ClientVisionSystem,
    pub floor_manager: FloorManager,
}

pub struct UIState {
    pub pilot_station_open: bool,
    pub pilot_station_id: Option<Uuid>,
    pub operating_mech_id: Option<Uuid>,
}

pub struct PlayerData {
    pub _id: Uuid,
    pub name: String,
    pub team: TeamId,
    pub location: PlayerLocation,
    pub carrying_resource: Option<ResourceType>,
}

pub struct MechState {
    pub id: Uuid,
    pub position: TilePos,
    pub world_position: WorldPos,
    pub team: TeamId,
    pub health: u32,
    pub shield: u32,
    pub upgrades: shared::MechUpgrades,
    pub floors: Vec<MechFloor>,
    pub _resource_inventory: HashMap<ResourceType, u32>,
}

pub struct MechFloor {
    pub _level: u8,
    // Note: Tiles are now rendered based on visible_tiles sent from server
    // The old local tile generation has been removed
    pub _ladder_positions: Vec<TilePos>, // Positions where you can move between floors
}

// Note: Old tile system has been replaced by the hybrid tile-entity system
// Use TileContent and TileVisual from shared::tile_entity instead

pub struct StationState {
    pub _id: Uuid,
    pub mech_id: Uuid,
    pub floor: u8,
    pub position: TilePos,
    pub station_type: StationType,
    pub occupied: bool,
    pub operated_by: Option<Uuid>,
}

pub struct ResourceState {
    pub id: Uuid,
    pub position: TilePos,
    pub resource_type: ResourceType,
}

pub struct ProjectileData {
    pub id: Uuid,
    pub position: WorldPos,
    pub _velocity: (f32, f32),
}

pub struct WeaponEffect {
    pub mech_id: Uuid,
    pub weapon_type: StationType,
    pub target: TilePos,
    pub timer: f32,
    pub _projectile_id: Option<Uuid>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            player_id: None,
            player_location: PlayerLocation::OutsideWorld(WorldPos::new(
                DEFAULT_SPAWN_CAMERA_MULTIPLIER * TILE_SIZE,
                DEFAULT_SPAWN_CAMERA_MULTIPLIER * TILE_SIZE,
            )),
            player_team: None,
            players: HashMap::new(),
            mechs: HashMap::new(),
            stations: HashMap::new(),
            resources: Vec::new(),
            projectiles: Vec::new(),
            weapon_effects: Vec::new(),
            camera_offset: (0.0, 0.0),
            ui_state: UIState {
                pilot_station_open: false,
                pilot_station_id: None,
                operating_mech_id: None,
            },
            visible_tiles: HashMap::new(),
            vision_system: ClientVisionSystem::new(),
            floor_manager: FloorManager::new(),
        }
    }

    pub fn update(&mut self, delta: f32) {
        // Update weapon effects
        self.weapon_effects.retain_mut(|effect| {
            effect.timer -= delta;
            effect.timer > 0.0
        });

        // Update vision system
        self.update_vision();

        // Update camera to follow player
        match &self.player_location {
            PlayerLocation::OutsideWorld(pos) => {
                self.camera_offset = (pos.x - screen_width() / 2.0, pos.y - screen_height() / 2.0);
            }
            PlayerLocation::InsideMech { pos, .. } => {
                // Center on the mech interior view
                self.camera_offset = (pos.x - screen_width() / 2.0, pos.y - screen_height() / 2.0);
            }
        }
    }

    /// Update the vision system using the new static method pattern
    pub fn update_vision(&mut self) {
        ClientVisionSystem::force_update(self);
    }
}

impl MechFloor {
    pub fn new(level: u8) -> Self {
        // Add ladders
        let mut ladder_positions = Vec::new();
        if level < (MECH_FLOORS - 1) as u8 {
            let ladder1 = TilePos::new(2, 2);
            let ladder2 = TilePos::new(FLOOR_WIDTH_TILES - 3, FLOOR_HEIGHT_TILES - 3);
            ladder_positions.push(ladder1);
            ladder_positions.push(ladder2);
        }

        Self {
            _level: level,
            _ladder_positions: ladder_positions,
        }
    }
}

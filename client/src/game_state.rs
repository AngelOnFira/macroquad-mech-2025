use shared::{types::*, constants::*, network_constants::*};
use std::collections::HashMap;
use uuid::Uuid;
use macroquad::prelude::*;

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
    pub transition: Option<TransitionState>,
}

pub struct UIState {
    pub pilot_station_open: bool,
    pub pilot_station_id: Option<Uuid>,
    pub operating_mech_id: Option<Uuid>,
}

pub struct TransitionState {
    pub _active: bool,
    pub transition_type: TransitionType,
    pub progress: f32, // 0.0 to 1.0
    pub from_location: PlayerLocation,
    pub to_location: PlayerLocation,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TransitionType {
    EnteringMech,
    ExitingMech,
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
    pub tiles: Vec<Vec<TileType>>,
    pub _ladder_positions: Vec<TilePos>, // Positions where you can move between floors
}

// TileType is now MechInteriorTile from shared
pub use shared::MechInteriorTile as TileType;

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
            player_location: PlayerLocation::OutsideWorld(WorldPos::new(DEFAULT_SPAWN_CAMERA_MULTIPLIER * TILE_SIZE, DEFAULT_SPAWN_CAMERA_MULTIPLIER * TILE_SIZE)),
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
            transition: None,
        }
    }

    pub fn update(&mut self, delta: f32) {
        // Update weapon effects
        self.weapon_effects.retain_mut(|effect| {
            effect.timer -= delta;
            effect.timer > 0.0
        });

        // Update transition
        if let Some(transition) = &mut self.transition {
            transition.progress += delta * 2.0; // 0.5 second transition
            if transition.progress >= 1.0 {
                self.transition = None;
            }
        }

        // Update camera to follow player
        let target_location = if let Some(transition) = &self.transition {
            // During transition, use target location for camera
            &transition.to_location
        } else {
            &self.player_location
        };

        match target_location {
            PlayerLocation::OutsideWorld(pos) => {
                self.camera_offset = (
                    pos.x - screen_width() / 2.0,
                    pos.y - screen_height() / 2.0,
                );
            }
            PlayerLocation::InsideMech { pos, .. } => {
                // Center on the mech interior view
                self.camera_offset = (
                    pos.x - screen_width() / 2.0,
                    pos.y - screen_height() / 2.0,
                );
            }
        }
    }
}

impl MechFloor {
    pub fn new(level: u8) -> Self {
        let mut tiles = vec![vec![TileType::Empty; FLOOR_WIDTH_TILES as usize]; FLOOR_HEIGHT_TILES as usize];
        
        // Create walls and floors
        for y in 0..FLOOR_HEIGHT_TILES {
            for x in 0..FLOOR_WIDTH_TILES {
                if x == 0 || x == FLOOR_WIDTH_TILES - 1 || y == 0 || y == FLOOR_HEIGHT_TILES - 1 {
                    tiles[y as usize][x as usize] = TileType::Wall;
                } else {
                    tiles[y as usize][x as usize] = TileType::Floor;
                }
            }
        }
        
        // Add ladders
        let mut ladder_positions = Vec::new();
        if level < (MECH_FLOORS - 1) as u8 {
            let ladder1 = TilePos::new(2, 2);
            let ladder2 = TilePos::new(FLOOR_WIDTH_TILES - 3, FLOOR_HEIGHT_TILES - 3);
            tiles[ladder1.y as usize][ladder1.x as usize] = TileType::Ladder;
            tiles[ladder2.y as usize][ladder2.x as usize] = TileType::Ladder;
            ladder_positions.push(ladder1);
            ladder_positions.push(ladder2);
        }
        
        // Add stations based on floor
        let station_positions = match level {
            0 => vec![
                (TilePos::new(5, 3), StationType::Engine),
                (TilePos::new(10, 3), StationType::Electrical),
                (TilePos::new(15, 3), StationType::Upgrade),
            ],
            1 => vec![
                (TilePos::new(5, 3), StationType::WeaponLaser),
                (TilePos::new(10, 3), StationType::WeaponProjectile),
                (TilePos::new(15, 3), StationType::Shield),
            ],
            2 => vec![
                (TilePos::new(8, 3), StationType::Repair),
            ],
            _ => vec![],
        };
        
        for (pos, station_type) in station_positions {
            tiles[pos.y as usize][pos.x as usize] = TileType::Station(station_type);
        }
        
        Self {
            _level: level,
            tiles,
            _ladder_positions: ladder_positions,
        }
    }
}
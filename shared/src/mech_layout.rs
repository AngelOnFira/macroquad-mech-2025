use crate::tile_entity::{StaticTile, FloorMap, TransitionType};
use crate::{
    uuid_gen::new_uuid, StationType, TilePos, FLOOR_HEIGHT_TILES, FLOOR_WIDTH_TILES,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Complete interior layout of a mech using HashMap-based floors
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MechInterior {
    pub floors: [FloorMap; 3], // Fixed 3 floors as per PRP specification
    pub current_occupants: HashMap<Uuid, u8>, // player_id -> floor_id
}

/// Multi-tile station size definition
#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub struct StationSize {
    pub width: u8,
    pub height: u8,
}

impl StationSize {
    pub const fn new(width: u8, height: u8) -> Self {
        Self { width, height }
    }
    
    pub const SINGLE: StationSize = StationSize::new(1, 1);
    pub const LARGE: StationSize = StationSize::new(2, 2);
    pub const WIDE: StationSize = StationSize::new(2, 1);
    pub const TALL: StationSize = StationSize::new(1, 2);
}

/// Station in a mech interior (enhanced with multi-tile support)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MechStation {
    pub id: Uuid,
    pub station_type: StationType,
    pub floor: u8,
    pub position: TilePos, // Top-left position for multi-tile stations
    pub size: StationSize,
    pub operated_by: Option<Uuid>,
}

/// Mech interior generator
pub struct MechLayoutGenerator;

impl MechLayoutGenerator {
    /// Create a complete mech interior with all floors and stations using HashMap-based system
    pub fn create_mech_interior(stations: &mut HashMap<Uuid, MechStation>) -> MechInterior {
        let mut floors = [FloorMap::new(), FloorMap::new(), FloorMap::new()];

        for floor_idx in 0..3 {
            // Generate basic floor layout (walls and floors)
            Self::generate_basic_floor_layout(&mut floors[floor_idx]);

            // Add cargo bay to floor 0
            if floor_idx == 0 {
                Self::add_cargo_bay_to_floor(&mut floors[floor_idx]);
            }

            // Add stairways between floors
            Self::add_stairways_to_floor(&mut floors[floor_idx], floor_idx as u8);

            // Add stations based on floor
            Self::add_stations_to_floor(&mut floors[floor_idx], stations, floor_idx as u8);
        }

        MechInterior { 
            floors, 
            current_occupants: HashMap::new(),
        }
    }

    /// Generate procedural floor layouts according to PRP specification
    pub fn generate_basic_floors() -> MechInterior {
        let mut stations = HashMap::new();
        
        let mut interior = MechInterior {
            floors: [FloorMap::new(), FloorMap::new(), FloorMap::new()],
            current_occupants: HashMap::new(),
        };
        
        // Generate floor 0 (engine room)
        Self::generate_basic_floor_layout(&mut interior.floors[0]);
        Self::place_station(&mut interior.floors[0], &mut stations, StationType::Engine, TilePos::new(4, 4), StationSize::LARGE, 0);
        Self::place_stairway(&mut interior.floors[0], TilePos::new(8, 8), 0, 1);
        
        // Generate floor 1 (bridge) 
        Self::generate_basic_floor_layout(&mut interior.floors[1]);
        Self::place_station(&mut interior.floors[1], &mut stations, StationType::Pilot, TilePos::new(4, 2), StationSize::WIDE, 1);
        Self::place_stairway(&mut interior.floors[1], TilePos::new(8, 8), 1, 0);
        Self::place_stairway(&mut interior.floors[1], TilePos::new(1, 1), 1, 2);
        
        // Generate floor 2 (weapons/shield)
        Self::generate_basic_floor_layout(&mut interior.floors[2]);
        Self::place_station(&mut interior.floors[2], &mut stations, StationType::WeaponLaser, TilePos::new(2, 2), StationSize::SINGLE, 2);
        Self::place_station(&mut interior.floors[2], &mut stations, StationType::Shield, TilePos::new(6, 6), StationSize::SINGLE, 2);
        Self::place_stairway(&mut interior.floors[2], TilePos::new(1, 1), 2, 1);
        
        interior
    }

    /// Generate basic floor layout with walls and floors using HashMap
    pub fn generate_basic_floor_layout(floor: &mut FloorMap) {
        for y in 0..FLOOR_HEIGHT_TILES {
            for x in 0..FLOOR_WIDTH_TILES {
                let pos = TilePos::new(x, y);
                if x == 0 || x == FLOOR_WIDTH_TILES - 1 || y == 0 || y == FLOOR_HEIGHT_TILES - 1 {
                    floor.static_tiles.insert(pos, StaticTile::MetalWall);
                } else {
                    floor.static_tiles.insert(pos, StaticTile::MetalFloor);
                }
            }
        }
    }

    /// Add cargo bay area to floor 0 using HashMap
    fn add_cargo_bay_to_floor(floor: &mut FloorMap) {
        // Create a 3x3 cargo bay area in the center-top of floor 0
        let cargo_x = FLOOR_WIDTH_TILES / 2 - 1;
        let cargo_y = 2; // Near the top of the floor

        for dy in 0..3 {
            for dx in 0..3 {
                let pos = TilePos::new(cargo_x + dx, cargo_y + dy);
                if pos.x >= 0 && pos.x < FLOOR_WIDTH_TILES && pos.y >= 0 && pos.y < FLOOR_HEIGHT_TILES {
                    floor.static_tiles.insert(pos, StaticTile::CargoFloor { wear: 0 });
                }
            }
        }
    }

    /// Add stairways to connect floors (new PRP system)
    fn add_stairways_to_floor(floor: &mut FloorMap, current_floor: u8) {
        match current_floor {
            0 => {
                // Floor 0: Only stair up to floor 1
                Self::place_stairway(floor, TilePos::new(8, 8), 0, 1);
            }
            1 => {
                // Floor 1: Stair down to floor 0 and stair up to floor 2
                Self::place_stairway(floor, TilePos::new(8, 8), 1, 0);
                Self::place_stairway(floor, TilePos::new(1, 1), 1, 2);
            }
            2 => {
                // Floor 2: Only stair down to floor 1
                Self::place_stairway(floor, TilePos::new(1, 1), 2, 1);
            }
            _ => {} // No stairways for invalid floors
        }
    }

    /// Place a stairway tile with target floor information
    pub fn place_stairway(floor: &mut FloorMap, pos: TilePos, current_floor: u8, target_floor: u8) {
        let transition_type = if target_floor > current_floor {
            TransitionType::StairUp { stage: 0, target_floor }
        } else {
            TransitionType::StairDown { stage: 0, target_floor }
        };
        
        floor.static_tiles.insert(pos, StaticTile::TransitionZone {
            zone_id: target_floor,
            transition_type,
        });
    }

    /// Place a multi-tile station on a floor
    pub fn place_station(
        floor: &mut FloorMap, 
        stations: &mut HashMap<Uuid, MechStation>, 
        station_type: StationType, 
        pos: TilePos, 
        size: StationSize, 
        floor_idx: u8
    ) {
        let station_id = new_uuid();
        
        // Create all tile positions for this station
        let mut positions = Vec::new();
        for dy in 0..size.height {
            for dx in 0..size.width {
                positions.push(TilePos::new(pos.x + dx as i32, pos.y + dy as i32));
            }
        }
        
        // Set the multi-tile station in the floor
        floor.set_multi_tile_station(&positions, station_id);
        
        // Create the station record
        let station = MechStation {
            id: station_id,
            station_type,
            floor: floor_idx,
            position: pos,
            size,
            operated_by: None,
        };
        
        stations.insert(station_id, station);
    }

    /// Add stations to a specific floor using new multi-tile system
    fn add_stations_to_floor(
        floor: &mut FloorMap,
        stations: &mut HashMap<Uuid, MechStation>,
        floor_idx: u8,
    ) {
        let floor_stations = Self::get_stations_for_floor(floor_idx);

        for (pos, station_type, size) in floor_stations {
            Self::place_station(floor, stations, station_type, pos, size, floor_idx);
        }
    }

    /// Get the list of stations for a specific floor with multi-tile sizes
    fn get_stations_for_floor(floor_idx: u8) -> Vec<(TilePos, StationType, StationSize)> {
        match floor_idx {
            0 => vec![
                // Floor 0 (Engine Room) - Large engine station
                (TilePos::new(4, 4), StationType::Engine, StationSize::LARGE),
            ],
            1 => vec![
                // Floor 1 (Bridge) - Command stations
                (TilePos::new(4, 2), StationType::Pilot, StationSize::WIDE),
                (TilePos::new(8, 4), StationType::Shield, StationSize::SINGLE),
                (TilePos::new(2, 6), StationType::Electrical, StationSize::SINGLE),
            ],
            2 => vec![
                // Floor 2 (Weapons/Operations)
                (TilePos::new(2, 2), StationType::WeaponLaser, StationSize::SINGLE),
                (TilePos::new(6, 2), StationType::WeaponProjectile, StationSize::SINGLE),
                (TilePos::new(4, 6), StationType::Repair, StationSize::WIDE),
                (TilePos::new(8, 8), StationType::Upgrade, StationSize::SINGLE),
            ],
            _ => vec![],
        }
    }

}

/// Implementation for MechInterior with convenience methods
impl MechInterior {
    /// Create a new empty mech interior
    pub fn new() -> Self {
        Self {
            floors: [FloorMap::new(), FloorMap::new(), FloorMap::new()],
            current_occupants: HashMap::new(),
        }
    }

    /// Set which floor a player is currently on
    pub fn set_player_floor(&mut self, player_id: Uuid, floor: u8) {
        if floor < 3 {
            self.current_occupants.insert(player_id, floor);
        }
    }

    /// Get which floor a player is on, if they're in this mech
    pub fn get_player_floor(&self, player_id: Uuid) -> Option<u8> {
        self.current_occupants.get(&player_id).copied()
    }

    /// Remove a player from the mech
    pub fn remove_player(&mut self, player_id: Uuid) {
        self.current_occupants.remove(&player_id);
    }

    /// Get all players on a specific floor
    pub fn get_players_on_floor(&self, floor: u8) -> Vec<Uuid> {
        if floor >= 3 {
            return Vec::new();
        }
        self.current_occupants
            .iter()
            .filter_map(|(player_id, player_floor)| {
                if *player_floor == floor {
                    Some(*player_id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get a specific floor
    pub fn get_floor(&self, floor_idx: u8) -> Option<&FloorMap> {
        if floor_idx < 3 {
            Some(&self.floors[floor_idx as usize])
        } else {
            None
        }
    }

    /// Get a mutable reference to a specific floor
    pub fn get_floor_mut(&mut self, floor_idx: u8) -> Option<&mut FloorMap> {
        if floor_idx < 3 {
            Some(&mut self.floors[floor_idx as usize])
        } else {
            None
        }
    }
}

impl Default for MechInterior {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mech_interior_generation() {
        let mut stations = HashMap::new();
        let interior = MechLayoutGenerator::create_mech_interior(&mut stations);

        assert_eq!(interior.floors.len(), 3); // Fixed 3 floors per PRP spec
        assert!(!stations.is_empty());

        // Check that each floor has tiles
        for floor in &interior.floors {
            // Should have walls around the perimeter
            assert!(floor.static_tiles.contains_key(&TilePos::new(0, 0)));
        }
    }

    #[test]
    fn test_basic_floors_generation() {
        let interior = MechLayoutGenerator::generate_basic_floors();

        assert_eq!(interior.floors.len(), 3);
        
        // Floor 0 should have an engine station
        let floor_0 = &interior.floors[0];
        assert!(floor_0.entity_tiles.values().any(|_| true)); // Has some entities
        
        // Check that stairways exist
        let has_stairway = floor_0.static_tiles.values().any(|tile| {
            matches!(tile, StaticTile::TransitionZone { .. })
        });
        assert!(has_stairway);
    }

    #[test]
    fn test_multi_tile_stations() {
        let mut stations = HashMap::new();
        let mut floor = FloorMap::new();
        
        MechLayoutGenerator::place_station(
            &mut floor, 
            &mut stations, 
            StationType::Engine, 
            TilePos::new(2, 2), 
            StationSize::LARGE, 
            0
        );

        assert_eq!(stations.len(), 1);
        let station = stations.values().next().unwrap();
        assert_eq!(station.size, StationSize::LARGE);
        assert_eq!(station.position, TilePos::new(2, 2));

        // Check that all 4 tiles (2x2) are occupied
        assert!(floor.multi_tile_stations.contains_key(&TilePos::new(2, 2)));
        assert!(floor.multi_tile_stations.contains_key(&TilePos::new(3, 2)));
        assert!(floor.multi_tile_stations.contains_key(&TilePos::new(2, 3)));
        assert!(floor.multi_tile_stations.contains_key(&TilePos::new(3, 3)));
    }

    #[test]
    fn test_stairway_placement() {
        let mut floor = FloorMap::new();
        MechLayoutGenerator::place_stairway(&mut floor, TilePos::new(5, 5), 0, 1);

        if let Some(StaticTile::TransitionZone { zone_id, transition_type }) = 
            floor.static_tiles.get(&TilePos::new(5, 5)) {
            assert_eq!(*zone_id, 1);
            match transition_type {
                TransitionType::StairUp { target_floor, .. } => {
                    assert_eq!(*target_floor, 1);
                }
                _ => panic!("Expected StairUp transition type"),
            }
        } else {
            panic!("Expected transition zone tile");
        }
    }

    #[test]
    fn test_mech_interior_player_management() {
        let mut interior = MechInterior::new();
        let player_id = Uuid::new_v4();

        // Set player on floor 1
        interior.set_player_floor(player_id, 1);
        assert_eq!(interior.get_player_floor(player_id), Some(1));

        // Get players on floor 1
        let players_on_floor_1 = interior.get_players_on_floor(1);
        assert_eq!(players_on_floor_1.len(), 1);
        assert_eq!(players_on_floor_1[0], player_id);

        // No players on floor 0
        let players_on_floor_0 = interior.get_players_on_floor(0);
        assert_eq!(players_on_floor_0.len(), 0);

        // Remove player
        interior.remove_player(player_id);
        assert_eq!(interior.get_player_floor(player_id), None);
    }
}

use std::collections::HashMap;
use uuid::Uuid;
use crate::{TilePos, StationType, FLOOR_WIDTH_TILES, FLOOR_HEIGHT_TILES, MECH_FLOORS, uuid_gen::new_uuid};
use crate::balance::STATION_POSITIONS;

/// Tile types for mech interiors
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TileType {
    Empty,
    Floor,
    Wall,
    Ladder,
    Station(StationType),
}

/// Layout of a single floor in a mech
#[derive(Clone, Debug)]
pub struct FloorLayout {
    pub tiles: Vec<Vec<TileType>>,
    pub ladders: Vec<TilePos>,
}

/// Complete interior layout of a mech
#[derive(Clone, Debug)]
pub struct MechInterior {
    pub floors: Vec<FloorLayout>,
}

/// Station in a mech interior (basic definition)
#[derive(Clone, Debug)]
pub struct MechStation {
    pub id: Uuid,
    pub station_type: StationType,
    pub floor: u8,
    pub position: TilePos,
    pub operated_by: Option<Uuid>,
}

/// Mech interior generator
pub struct MechLayoutGenerator;

impl MechLayoutGenerator {
    /// Create a complete mech interior with all floors and stations
    pub fn create_mech_interior(stations: &mut HashMap<Uuid, MechStation>) -> MechInterior {
        let mut floors = Vec::new();

        for floor_idx in 0..MECH_FLOORS {
            let mut tiles = vec![vec![TileType::Empty; FLOOR_WIDTH_TILES as usize]; FLOOR_HEIGHT_TILES as usize];
            let mut ladders = Vec::new();

            // Create walls and floors
            Self::generate_basic_floor_layout(&mut tiles);
            
            // Add ladders between floors
            if floor_idx < MECH_FLOORS - 1 {
                Self::add_ladders_to_floor(&mut tiles, &mut ladders);
            }

            // Add stations based on floor
            Self::add_stations_to_floor(&mut tiles, stations, floor_idx);

            floors.push(FloorLayout { tiles, ladders });
        }

        MechInterior { floors }
    }
    
    /// Generate basic floor layout with walls and floors
    fn generate_basic_floor_layout(tiles: &mut Vec<Vec<TileType>>) {
        for y in 0..FLOOR_HEIGHT_TILES {
            for x in 0..FLOOR_WIDTH_TILES {
                if x == 0 || x == FLOOR_WIDTH_TILES - 1 || y == 0 || y == FLOOR_HEIGHT_TILES - 1 {
                    tiles[y as usize][x as usize] = TileType::Wall;
                } else {
                    tiles[y as usize][x as usize] = TileType::Floor;
                }
            }
        }
    }
    
    /// Add ladders to connect floors
    fn add_ladders_to_floor(tiles: &mut Vec<Vec<TileType>>, ladders: &mut Vec<TilePos>) {
        let ladder1 = TilePos::new(2, 2);
        let ladder2 = TilePos::new(FLOOR_WIDTH_TILES - 3, FLOOR_HEIGHT_TILES - 3);
        tiles[ladder1.y as usize][ladder1.x as usize] = TileType::Ladder;
        tiles[ladder2.y as usize][ladder2.x as usize] = TileType::Ladder;
        ladders.push(ladder1);
        ladders.push(ladder2);
    }
    
    /// Add stations to a specific floor
    fn add_stations_to_floor(
        tiles: &mut Vec<Vec<TileType>>, 
        stations: &mut HashMap<Uuid, MechStation>, 
        floor_idx: usize
    ) {
        let floor_stations = Self::get_stations_for_floor(floor_idx);

        for (pos, station_type) in floor_stations {
            tiles[pos.y as usize][pos.x as usize] = TileType::Station(station_type);
            let station = MechStation {
                id: new_uuid(),
                station_type,
                floor: floor_idx as u8,
                position: pos,
                operated_by: None,
            };
            stations.insert(station.id, station);
        }
    }
    
    /// Get the list of stations for a specific floor
    fn get_stations_for_floor(floor_idx: usize) -> Vec<(TilePos, StationType)> {
        match floor_idx {
            0 => vec![
                (TilePos::new(STATION_POSITIONS[0][0].0, STATION_POSITIONS[0][0].1), StationType::Engine),
                (TilePos::new(STATION_POSITIONS[0][1].0, STATION_POSITIONS[0][1].1), StationType::Electrical),
                (TilePos::new(STATION_POSITIONS[0][2].0, STATION_POSITIONS[0][2].1), StationType::Upgrade),
            ],
            1 => vec![
                (TilePos::new(STATION_POSITIONS[1][0].0, STATION_POSITIONS[1][0].1), StationType::WeaponLaser),
                (TilePos::new(STATION_POSITIONS[1][1].0, STATION_POSITIONS[1][1].1), StationType::WeaponProjectile),
                (TilePos::new(STATION_POSITIONS[1][2].0, STATION_POSITIONS[1][2].1), StationType::Shield),
            ],
            2 => vec![
                (TilePos::new(STATION_POSITIONS[2][0].0, STATION_POSITIONS[2][0].1), StationType::Repair),
            ],
            _ => vec![],
        }
    }
    
    /// Create a custom floor layout (for future expansion)
    pub fn create_custom_floor(
        width: i32, 
        height: i32, 
        station_configs: Vec<(TilePos, StationType)>
    ) -> (FloorLayout, HashMap<Uuid, MechStation>) {
        let mut tiles = vec![vec![TileType::Empty; width as usize]; height as usize];
        let mut stations = HashMap::new();
        let ladders = Vec::new();
        
        // Create basic layout
        for y in 0..height {
            for x in 0..width {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    tiles[y as usize][x as usize] = TileType::Wall;
                } else {
                    tiles[y as usize][x as usize] = TileType::Floor;
                }
            }
        }
        
        // Add custom stations
        for (pos, station_type) in station_configs {
            if pos.x >= 0 && pos.x < width && pos.y >= 0 && pos.y < height {
                tiles[pos.y as usize][pos.x as usize] = TileType::Station(station_type);
                let station = MechStation {
                    id: new_uuid(),
                    station_type,
                    floor: 0,
                    position: pos,
                    operated_by: None,
                };
                stations.insert(station.id, station);
            }
        }
        
        (FloorLayout { tiles, ladders }, stations)
    }
}

impl FloorLayout {
    /// Check if a position is walkable on this floor
    pub fn is_walkable(&self, pos: TilePos) -> bool {
        if pos.x < 0 || pos.x >= FLOOR_WIDTH_TILES || pos.y < 0 || pos.y >= FLOOR_HEIGHT_TILES {
            return false;
        }
        
        let tile = &self.tiles[pos.y as usize][pos.x as usize];
        match tile {
            TileType::Wall | TileType::Empty => false,
            _ => true,
        }
    }
    
    /// Get the tile type at a position
    pub fn get_tile(&self, pos: TilePos) -> Option<&TileType> {
        if pos.x < 0 || pos.x >= FLOOR_WIDTH_TILES || pos.y < 0 || pos.y >= FLOOR_HEIGHT_TILES {
            return None;
        }
        
        Some(&self.tiles[pos.y as usize][pos.x as usize])
    }
    
    /// Check if there's a ladder at this position
    pub fn has_ladder(&self, pos: TilePos) -> bool {
        self.ladders.contains(&pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mech_interior_generation() {
        let mut stations = HashMap::new();
        let interior = MechLayoutGenerator::create_mech_interior(&mut stations);
        
        assert_eq!(interior.floors.len(), MECH_FLOORS);
        assert!(!stations.is_empty());
        
        // Check that each floor has the right dimensions
        for floor in &interior.floors {
            assert_eq!(floor.tiles.len(), FLOOR_HEIGHT_TILES as usize);
            assert_eq!(floor.tiles[0].len(), FLOOR_WIDTH_TILES as usize);
        }
    }
    
    #[test]
    fn test_floor_walkability() {
        let mut stations = HashMap::new();
        let interior = MechLayoutGenerator::create_mech_interior(&mut stations);
        
        let floor = &interior.floors[0];
        
        // Corner should be a wall (not walkable)
        assert!(!floor.is_walkable(TilePos::new(0, 0)));
        
        // Center should be walkable
        assert!(floor.is_walkable(TilePos::new(FLOOR_WIDTH_TILES / 2, FLOOR_HEIGHT_TILES / 2)));
        
        // Out of bounds should not be walkable
        assert!(!floor.is_walkable(TilePos::new(-1, 0)));
        assert!(!floor.is_walkable(TilePos::new(FLOOR_WIDTH_TILES, 0)));
    }
    
    #[test]
    fn test_custom_floor_creation() {
        let station_configs = vec![
            (TilePos::new(2, 2), StationType::Engine),
            (TilePos::new(4, 4), StationType::WeaponLaser),
        ];
        
        let (floor, stations) = MechLayoutGenerator::create_custom_floor(10, 10, station_configs);
        
        assert_eq!(stations.len(), 2);
        assert_eq!(floor.tiles.len(), 10);
        assert_eq!(floor.tiles[0].len(), 10);
        
        // Check stations are placed correctly
        assert_eq!(floor.get_tile(TilePos::new(2, 2)), Some(&TileType::Station(StationType::Engine)));
        assert_eq!(floor.get_tile(TilePos::new(4, 4)), Some(&TileType::Station(StationType::WeaponLaser)));
    }
}
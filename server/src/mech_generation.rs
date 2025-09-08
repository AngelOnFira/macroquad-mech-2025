use shared::{
    mech_layout::{MechInterior, MechLayoutGenerator, MechStation, StationSize},
    StationType, TilePos, TeamId,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Procedural mech floor generation system for the server
pub struct MechGenerator;

impl MechGenerator {
    /// Generate a complete mech with procedural floor layouts and stations
    pub fn generate_mech_interior(team: TeamId) -> (MechInterior, HashMap<Uuid, MechStation>) {
        let mut stations = HashMap::new();
        
        // Use the shared layout generator for basic floor generation
        let interior = MechLayoutGenerator::create_mech_interior(&mut stations);
        
        // Customize based on team (future expansion)
        Self::customize_for_team(&interior, &mut stations, team);
        
        (interior, stations)
    }

    /// Generate example floor layouts according to PRP specification
    pub fn generate_example_layouts() -> (MechInterior, HashMap<Uuid, MechStation>) {
        let mut stations = HashMap::new();
        
        // Use the PRP-specified generation method
        let mut interior = MechLayoutGenerator::generate_basic_floors();
        
        // Add some additional stations for a complete example
        if let Some(floor_1) = interior.get_floor_mut(1) {
            MechLayoutGenerator::place_station(
                floor_1, 
                &mut stations, 
                StationType::WeaponProjectile, 
                TilePos::new(7, 3), 
                StationSize::SINGLE, 
                1
            );
        }
        
        if let Some(floor_2) = interior.get_floor_mut(2) {
            MechLayoutGenerator::place_station(
                floor_2, 
                &mut stations, 
                StationType::Electrical, 
                TilePos::new(2, 8), 
                StationSize::SINGLE, 
                2
            );
        }
        
        (interior, stations)
    }

    /// Generate a custom mech layout for specific purposes (testing, special events, etc.)
    pub fn generate_custom_layout(config: MechGenerationConfig) -> (MechInterior, HashMap<Uuid, MechStation>) {
        let mut stations = HashMap::new();
        let mut interior = MechInterior::new();
        
        // Generate floors based on configuration
        for (floor_idx, floor_config) in config.floor_configs.iter().enumerate() {
            if let Some(floor) = interior.get_floor_mut(floor_idx as u8) {
                // Generate basic floor layout
                MechLayoutGenerator::generate_basic_floor_layout(floor);
                
                // Add configured stations
                for station_config in &floor_config.stations {
                    MechLayoutGenerator::place_station(
                        floor,
                        &mut stations,
                        station_config.station_type,
                        station_config.position,
                        station_config.size,
                        floor_idx as u8,
                    );
                }
                
                // Add configured stairways
                for stairway in &floor_config.stairways {
                    MechLayoutGenerator::place_stairway(
                        floor,
                        stairway.position,
                        floor_idx as u8,
                        stairway.target_floor,
                    );
                }
            }
        }
        
        (interior, stations)
    }

    /// Validate that a generated mech meets all requirements
    pub fn validate_mech_interior(interior: &MechInterior, stations: &HashMap<Uuid, MechStation>) -> Result<(), MechValidationError> {
        // Check that all 3 floors exist
        if interior.floors.len() != 3 {
            return Err(MechValidationError::InvalidFloorCount);
        }

        // Check that all floors have proper connectivity via stairways
        for floor_idx in 0..3 {
            let floor = &interior.floors[floor_idx];
            let has_stairway = floor.static_tiles.values().any(|tile| {
                matches!(tile, shared::tile_entity::StaticTile::TransitionZone { .. })
            });
            
            // Floor 0 and 2 should have at least one stairway, floor 1 should have two
            let required_stairways = match floor_idx {
                0 | 2 => 1,
                1 => 2,
                _ => 0,
            };
            
            if required_stairways > 0 && !has_stairway {
                return Err(MechValidationError::MissingStairways { floor: floor_idx as u8 });
            }
        }

        // Check that all stations are placed within floor boundaries
        for station in stations.values() {
            if station.floor >= 3 {
                return Err(MechValidationError::StationOutOfBounds { 
                    station_id: station.id, 
                    floor: station.floor 
                });
            }
            
            if !station.position.is_in_mech_floor_bounds() {
                return Err(MechValidationError::StationOutOfBounds { 
                    station_id: station.id, 
                    floor: station.floor 
                });
            }
        }

        // Check that multi-tile stations don't overlap
        for station in stations.values() {
            if station.size.width > 1 || station.size.height > 1 {
                // Check that all tiles of this station fit within bounds
                for dy in 0..station.size.height {
                    for dx in 0..station.size.width {
                        let pos = TilePos::new(
                            station.position.x + dx as i32, 
                            station.position.y + dy as i32
                        );
                        if !pos.is_in_mech_floor_bounds() {
                            return Err(MechValidationError::StationOutOfBounds { 
                                station_id: station.id, 
                                floor: station.floor 
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Customize mech layout based on team (future expansion)
    fn customize_for_team(_interior: &MechInterior, _stations: &mut HashMap<Uuid, MechStation>, _team: TeamId) {
        // Future: Different teams could have different mech layouts
        // For now, all teams use the same basic layout
    }
}

/// Configuration for custom mech generation
#[derive(Debug, Clone)]
pub struct MechGenerationConfig {
    pub floor_configs: Vec<FloorGenerationConfig>,
}

#[derive(Debug, Clone)]
pub struct FloorGenerationConfig {
    pub stations: Vec<StationGenerationConfig>,
    pub stairways: Vec<StairwayGenerationConfig>,
}

#[derive(Debug, Clone)]
pub struct StationGenerationConfig {
    pub station_type: StationType,
    pub position: TilePos,
    pub size: StationSize,
}

#[derive(Debug, Clone)]
pub struct StairwayGenerationConfig {
    pub position: TilePos,
    pub target_floor: u8,
}

/// Errors that can occur during mech validation
#[derive(Debug, Clone)]
pub enum MechValidationError {
    InvalidFloorCount,
    MissingStairways { floor: u8 },
    StationOutOfBounds { station_id: Uuid, floor: u8 },
    StairwayConnectivityIssue { floor: u8 },
}

impl std::fmt::Display for MechValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MechValidationError::InvalidFloorCount => write!(f, "Mech must have exactly 3 floors"),
            MechValidationError::MissingStairways { floor } => write!(f, "Floor {} is missing required stairways", floor),
            MechValidationError::StationOutOfBounds { station_id, floor } => {
                write!(f, "Station {} on floor {} is placed out of bounds", station_id, floor)
            }
            MechValidationError::StairwayConnectivityIssue { floor } => {
                write!(f, "Floor {} has stairway connectivity issues", floor)
            }
        }
    }
}

impl std::error::Error for MechValidationError {}

/// Get the appropriate size for a station type
pub fn get_station_size(station_type: StationType) -> StationSize {
    match station_type {
        StationType::Engine => StationSize::LARGE, // 2x2
        StationType::Pilot => StationSize::WIDE,   // 2x1
        StationType::Repair => StationSize::WIDE,  // 2x1
        StationType::WeaponLaser | StationType::WeaponProjectile => StationSize::SINGLE, // 1x1
        StationType::Shield | StationType::Electrical | StationType::Upgrade => StationSize::SINGLE, // 1x1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mech_interior() {
        let (interior, stations) = MechGenerator::generate_mech_interior(TeamId::Red);
        
        assert_eq!(interior.floors.len(), 3);
        assert!(!stations.is_empty());
        
        // Validate the generated mech
        assert!(MechGenerator::validate_mech_interior(&interior, &stations).is_ok());
    }

    #[test]
    fn test_generate_example_layouts() {
        let (interior, stations) = MechGenerator::generate_example_layouts();
        
        assert_eq!(interior.floors.len(), 3);
        assert!(!stations.is_empty());
        
        // Should have stations on multiple floors
        let floor_counts = stations.values().fold([0u32; 3], |mut counts, station| {
            if station.floor < 3 {
                counts[station.floor as usize] += 1;
            }
            counts
        });
        
        assert!(floor_counts[0] > 0, "Floor 0 should have stations");
        assert!(floor_counts[1] > 0, "Floor 1 should have stations");
        assert!(floor_counts[2] > 0, "Floor 2 should have stations");
    }

    #[test]
    fn test_custom_mech_generation() {
        let config = MechGenerationConfig {
            floor_configs: vec![
                FloorGenerationConfig {
                    stations: vec![
                        StationGenerationConfig {
                            station_type: StationType::Engine,
                            position: TilePos::new(3, 3),
                            size: StationSize::LARGE,
                        }
                    ],
                    stairways: vec![
                        StairwayGenerationConfig {
                            position: TilePos::new(8, 8),
                            target_floor: 1,
                        }
                    ],
                },
                FloorGenerationConfig {
                    stations: vec![
                        StationGenerationConfig {
                            station_type: StationType::Pilot,
                            position: TilePos::new(5, 5),
                            size: StationSize::WIDE,
                        }
                    ],
                    stairways: vec![],
                },
                FloorGenerationConfig {
                    stations: vec![],
                    stairways: vec![],
                },
            ],
        };

        let (interior, stations) = MechGenerator::generate_custom_layout(config);
        
        assert_eq!(interior.floors.len(), 3);
        assert_eq!(stations.len(), 2); // Engine + Pilot stations
        
        // Find the engine station
        let engine_station = stations.values().find(|s| s.station_type == StationType::Engine).unwrap();
        assert_eq!(engine_station.floor, 0);
        assert_eq!(engine_station.size, StationSize::LARGE);
    }

    #[test]
    fn test_mech_validation() {
        // Test valid mech
        let (interior, stations) = MechGenerator::generate_example_layouts();
        assert!(MechGenerator::validate_mech_interior(&interior, &stations).is_ok());

        // Test invalid station placement
        let mut invalid_stations = HashMap::new();
        invalid_stations.insert(Uuid::new_v4(), MechStation {
            id: Uuid::new_v4(),
            station_type: StationType::Engine,
            floor: 5, // Invalid floor
            position: TilePos::new(0, 0),
            size: StationSize::SINGLE,
            operated_by: None,
        });

        let result = MechGenerator::validate_mech_interior(&interior, &invalid_stations);
        assert!(result.is_err());
        match result.err().unwrap() {
            MechValidationError::StationOutOfBounds { floor, .. } => assert_eq!(floor, 5),
            _ => panic!("Expected StationOutOfBounds error"),
        }
    }
}
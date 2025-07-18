use shared::{
    tile_entity::*, components::*, vision::*, tile_migration::*,
    TilePos, WorldPos, Direction, StationType, TeamId, ResourceType,
    constants::*, mech_layout::MechInterior
};
use crate::entity_storage::EntityStorage;
use uuid::Uuid;
use std::collections::HashMap;

/// Integration layer between the new hybrid tile-entity system and the existing server
pub struct HybridGameWorld {
    // Tile map for the world
    pub tile_map: TileMap,
    
    // Entity storage for complex objects
    pub entities: EntityStorage,
    
    // Vision system for calculating visibility
    pub vision_system: VisionSystem,
    
    // Mapping from old mech IDs to entity IDs
    mech_entities: HashMap<Uuid, Uuid>,
    
    // Station registry for creating station entities
    station_registry: shared::StationRegistry,
}

impl HybridGameWorld {
    pub fn new() -> Self {
        Self {
            tile_map: TileMap::new(),
            entities: EntityStorage::new(),
            vision_system: VisionSystem::new(),
            mech_entities: HashMap::new(),
            station_registry: shared::StationRegistry::new(),
        }
    }
    
    /// Initialize the world with grass tiles
    pub fn initialize_world(&mut self) {
        // Fill world with grass (empty tiles in new system)
        for x in 0..ARENA_WIDTH_TILES {
            for y in 0..ARENA_HEIGHT_TILES {
                let pos = TilePos::new(x as i32, y as i32);
                // Grass is represented as Empty in the new system
                // We don't need to explicitly set it
            }
        }
    }
    
    /// Create a mech using the hybrid system
    pub fn create_mech(
        &mut self, 
        position: TilePos, 
        team: TeamId,
        interior: &MechInterior,
    ) -> (Uuid, Uuid) {
        let mech_id = Uuid::new_v4();
        let mech_entity_id = self.entities.create_entity(format!("{:?} Mech", team));
        
        // Add mech as an entity with position
        let mech_pos = Position {
            tile: position,
            world: position.to_world(),
            floor: None,
            mech_id: None,
        };
        self.entities.update_position(mech_entity_id, mech_pos);
        
        // Create mech tile map
        let mut mech_tile_map = MechTileMap::new(mech_entity_id, MECH_FLOORS);
        
        // Set up mech interior tiles
        for (floor_idx, floor) in interior.floors.iter().enumerate() {
            if let Some(floor_map) = mech_tile_map.get_floor_mut(floor_idx) {
                // Add floor tiles
                for y in 0..FLOOR_HEIGHT_TILES {
                    for x in 0..FLOOR_WIDTH_TILES {
                        let local_pos = TilePos::new(x as i32, y as i32);
                        
                        // Check if it's a wall position
                        let is_wall = x == 0 || x == FLOOR_WIDTH_TILES - 1 ||
                                     y == 0 || y == FLOOR_HEIGHT_TILES - 1;
                        
                        if is_wall {
                            floor_map.set_static_tile(local_pos, StaticTile::MetalWall);
                        } else {
                            floor_map.set_static_tile(local_pos, StaticTile::MetalFloor);
                        }
                    }
                }
                
                // Add windows on upper floors
                if floor_idx > 0 {
                    // Front window
                    floor_map.set_static_tile(
                        TilePos::new(FLOOR_WIDTH_TILES / 2, 0),
                        StaticTile::Window { facing: Direction::Up }
                    );
                    // Side windows
                    floor_map.set_static_tile(
                        TilePos::new(0, FLOOR_HEIGHT_TILES / 2),
                        StaticTile::Window { facing: Direction::Left }
                    );
                    floor_map.set_static_tile(
                        TilePos::new(FLOOR_WIDTH_TILES - 1, FLOOR_HEIGHT_TILES / 2),
                        StaticTile::Window { facing: Direction::Right }
                    );
                }
                
                // Add ladders
                for ladder_pos in &floor.ladders {
                    floor_map.set_static_tile(*ladder_pos, StaticTile::TransitionZone {
                        zone_id: floor_idx as u8,
                        transition_type: TransitionType::Ladder,
                    });
                }
            }
        }
        
        // Add to tile map
        self.tile_map.mech_tiles.insert(mech_id, mech_tile_map);
        
        // Create door transitions in world
        let door_position = TilePos::new(
            position.x + (FLOOR_WIDTH_TILES / 2 - 1),
            position.y + FLOOR_HEIGHT_TILES - 1
        );
        
        let transitions = TileMigration::create_door_transitions(
            mech_id,
            &[(door_position, Direction::Down)]
        );
        
        for (pos, tile) in transitions {
            self.tile_map.set_static_tile(pos, tile);
        }
        
        self.mech_entities.insert(mech_id, mech_entity_id);
        (mech_id, mech_entity_id)
    }
    
    /// Create station entities for a mech
    pub fn create_stations_for_mech(
        &mut self,
        mech_id: Uuid,
        stations: &[(u8, TilePos, StationType)], // (floor, position, type)
    ) -> Vec<(Uuid, Uuid)> { // Returns (station_instance_id, entity_id)
        let mut created_stations = Vec::new();
        
        for (floor, pos, station_type) in stations {
            // Create entity template
            let template = create_station_template(*station_type);
            
            // Create position for the station
            let station_pos = Position {
                tile: *pos,
                world: pos.to_world(),
                floor: Some(*floor),
                mech_id: Some(mech_id),
            };
            
            // Spawn entity
            let entity_id = self.entities.spawn_from_template(&template, station_pos);
            
            // Create station instance for game logic
            let station_instance = self.station_registry.create_station(
                *station_type,
                *floor,
                *pos
            ).expect("Failed to create station instance");
            
            created_stations.push((station_instance.id, entity_id));
            
            // Add entity reference to mech tile map
            if let Some(mech_tiles) = self.tile_map.mech_tiles.get_mut(&mech_id) {
                if let Some(floor_map) = mech_tiles.get_floor_mut(*floor as usize) {
                    floor_map.set_entity_tile(*pos, entity_id);
                }
            }
        }
        
        created_stations
    }
    
    /// Update mech position in the world
    pub fn update_mech_position(&mut self, mech_id: Uuid, new_position: TilePos) {
        if let Some(&entity_id) = self.mech_entities.get(&mech_id) {
            let new_pos = Position {
                tile: new_position,
                world: new_position.to_world(),
                floor: None,
                mech_id: None,
            };
            self.entities.update_position(entity_id, new_pos);
        }
    }
    
    /// Get visibility for a player
    pub fn get_player_visibility(&mut self, player_id: Uuid, player_pos: WorldPos) -> &VisibilityData {
        self.vision_system.calculate_visibility(
            player_id,
            player_pos,
            MAX_DISTANCE_FROM_MECH * TILE_SIZE,
            &self.tile_map,
            &self.entities,
        )
    }
    
    /// Check if movement is valid
    pub fn can_move_to(&self, entity_id: Uuid, target_pos: WorldPos) -> bool {
        handle_movement(&self.tile_map, &self.entities, entity_id, target_pos).is_ok()
    }
    
    /// Get tile at world position
    pub fn get_tile_at(&self, pos: WorldPos) -> Option<TileContent> {
        self.tile_map.get_tile_at(pos)
    }
    
    /// Get all stations in a mech
    pub fn get_mech_stations(&self, mech_id: Uuid) -> Vec<(Uuid, &Station, &Position)> {
        self.entities.get_stations_in_mech(mech_id)
    }
}

// Re-export for convenience
pub use shared::tile_entity::TileMap;
pub use shared::components::ComponentStorage;
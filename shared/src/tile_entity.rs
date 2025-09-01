use crate::{Direction, StationType, TilePos, WorldPos};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// =============================================================================
// Core Tile Content Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileContent {
    Empty,
    Static(StaticTile),
    Entity(Uuid), // EntityId reference
}

// Simple tiles that don't need complex behavior
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StaticTile {
    // World tiles
    Grass,
    Rock,

    // Floors
    MetalFloor,
    CargoFloor {
        wear: u8,
    },

    // Walls
    MetalWall,
    ReinforcedWall,

    // Windows
    Window {
        facing: Direction,
    },
    ReinforcedWindow {
        facing: Direction,
        tint: WindowTint,
    },

    // Transitions
    TransitionZone {
        zone_id: u8,
        transition_type: TransitionType,
    },

    // Basic infrastructure
    PowerConduit,
    DataCable,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WindowTint {
    Clear,
    Tinted,
    Darkened,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TransitionType {
    MechEntrance { stage: u8 }, // 0 = first tile, 1 = second tile
    StairUp { stage: u8 },
    StairDown { stage: u8 },
    Ladder,
}

// =============================================================================
// Tile Map Structure
// =============================================================================

pub struct TileMap {
    // Simple tiles indexed by position
    pub static_tiles: HashMap<TilePos, StaticTile>,

    // Entity references for complex tiles
    pub entity_tiles: HashMap<TilePos, Uuid>,

    // Spatial index for fast lookups
    pub spatial_index: SpatialIndex,

    // Mech-relative tiles
    pub mech_tiles: HashMap<Uuid, MechTileMap>,
}

pub struct MechTileMap {
    pub floors: Vec<FloorMap>,
    pub position: TilePos, // World position of mech
}

#[derive(Clone)]
pub struct FloorMap {
    pub static_tiles: HashMap<TilePos, StaticTile>,
    pub entity_tiles: HashMap<TilePos, Uuid>,
}

impl FloorMap {
    pub fn new() -> Self {
        Self {
            static_tiles: HashMap::new(),
            entity_tiles: HashMap::new(),
        }
    }

    pub fn get_tile(&self, pos: TilePos) -> Option<TileContent> {
        if let Some(entity_id) = self.entity_tiles.get(&pos) {
            Some(TileContent::Entity(*entity_id))
        } else if let Some(static_tile) = self.static_tiles.get(&pos) {
            Some(TileContent::Static(*static_tile))
        } else {
            Some(TileContent::Empty)
        }
    }
}

// Simplified spatial index for now
pub struct SpatialIndex {
    // TODO: Implement efficient spatial queries
    _data: Vec<u8>,
}

// =============================================================================
// Static Tile Behaviors
// =============================================================================

impl StaticTile {
    pub fn is_walkable(&self) -> bool {
        match self {
            StaticTile::Grass => true,
            StaticTile::Rock => false,
            StaticTile::MetalFloor | StaticTile::CargoFloor { .. } => true,
            StaticTile::TransitionZone { .. } => true,
            StaticTile::PowerConduit | StaticTile::DataCable => true,
            _ => false,
        }
    }

    pub fn blocks_vision(&self) -> bool {
        match self {
            StaticTile::Rock => true,
            StaticTile::MetalWall | StaticTile::ReinforcedWall => true,
            _ => false,
        }
    }

    pub fn vision_attenuation(&self) -> f32 {
        match self {
            StaticTile::Window { .. } => 0.2,
            StaticTile::ReinforcedWindow { .. } => 0.3,
            StaticTile::MetalWall | StaticTile::ReinforcedWall => 1.0,
            _ => 0.0,
        }
    }

    pub fn on_enter(&self, actor: Uuid) -> Option<TileEvent> {
        match self {
            StaticTile::TransitionZone {
                zone_id,
                transition_type,
            } => Some(TileEvent::BeginTransition {
                actor,
                zone_id: *zone_id,
                transition_type: *transition_type,
            }),
            _ => None,
        }
    }
}

// =============================================================================
// Tile Events
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileEvent {
    BeginTransition {
        actor: Uuid,
        zone_id: u8,
        transition_type: TransitionType,
    },
    CompleteTransition {
        actor: Uuid,
    },
    TileChanged {
        pos: TilePos,
        old_tile: TileContent,
        new_tile: TileContent,
    },
    // New behavior events
    ProximityTriggered {
        entity: Uuid,
        actor: Uuid,
        distance: f32,
    },
    ResourcePickedUp {
        resource_entity: Uuid,
        actor: Uuid,
        resource_type: crate::ResourceType,
    },
    InteractionStarted {
        entity: Uuid,
        actor: Uuid,
        interaction_type: String,
    },
    ShowInteractionPrompt {
        entity: Uuid,
        actor: Uuid,
        prompt: String,
    },
    AutoInteractionTriggered {
        entity: Uuid,
        actor: Uuid,
        action: crate::components::AutoInteractionType,
    },
    MechEntered {
        mech_id: Uuid,
        actor: Uuid,
        floor: u8,
    },
    ResourceDropped {
        actor: Uuid,
        resource_type: crate::ResourceType,
        position: TilePos,
    },
}

// =============================================================================
// Client Representation
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTile {
    pub visual: TileVisual,
    pub walkable: bool, // For prediction
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileVisual {
    // Static visuals
    Floor {
        material: Material,
        wear: u8,
    },
    Wall {
        material: Material,
    },
    Window {
        broken: bool,
        facing: Direction,
    },

    // Entity visuals
    Station {
        station_type: StationType,
        active: bool,
    },
    Turret {
        facing: Direction,
        firing: bool,
    },

    // Effects
    TransitionFade {
        progress: f32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Material {
    Metal,
    Reinforced,
    Damaged,
}

// =============================================================================
// Tile Map Implementation
// =============================================================================

impl TileMap {
    pub fn new() -> Self {
        Self {
            static_tiles: HashMap::new(),
            entity_tiles: HashMap::new(),
            spatial_index: SpatialIndex { _data: vec![] },
            mech_tiles: HashMap::new(),
        }
    }

    pub fn set_world_tile(&mut self, pos: TilePos, content: TileContent) {
        match content {
            TileContent::Empty => {
                self.static_tiles.remove(&pos);
                self.entity_tiles.remove(&pos);
            }
            TileContent::Static(tile) => {
                self.static_tiles.insert(pos, tile);
                self.entity_tiles.remove(&pos);
            }
            TileContent::Entity(id) => {
                self.entity_tiles.insert(pos, id);
                self.static_tiles.remove(&pos);
            }
        }
    }

    pub fn create_mech(&mut self, mech_id: Uuid, position: TilePos) -> &mut MechTileMap {
        self.mech_tiles.entry(mech_id).or_insert_with(|| {
            MechTileMap {
                floors: vec![FloorMap::new(); 3], // 3 floors
                position,
            }
        })
    }

    pub fn get_world_tile(&self, pos: TilePos) -> Option<TileContent> {
        if let Some(entity_id) = self.entity_tiles.get(&pos) {
            Some(TileContent::Entity(*entity_id))
        } else if let Some(static_tile) = self.static_tiles.get(&pos) {
            Some(TileContent::Static(*static_tile))
        } else {
            None
        }
    }

    // Get tile at world position, accounting for mechs
    pub fn get_tile_at(&self, world_pos: WorldPos) -> Option<TileContent> {
        // First check if we're inside a mech
        if let Some((mech_id, local_pos)) = self.world_to_mech_local(world_pos) {
            self.mech_tiles
                .get(&mech_id)
                .and_then(|mech| {
                    // Determine floor based on local position
                    let floor_idx = 0; // TODO: Calculate based on position
                    mech.floors.get(floor_idx)
                })
                .and_then(|floor| {
                    let tile_pos = local_pos.to_tile();
                    if let Some(entity_id) = floor.entity_tiles.get(&tile_pos) {
                        Some(TileContent::Entity(*entity_id))
                    } else if let Some(static_tile) = floor.static_tiles.get(&tile_pos) {
                        Some(TileContent::Static(*static_tile))
                    } else {
                        Some(TileContent::Empty)
                    }
                })
        } else {
            // Otherwise check world tiles
            let tile_pos = world_pos.to_tile();
            if let Some(entity_id) = self.entity_tiles.get(&tile_pos) {
                Some(TileContent::Entity(*entity_id))
            } else if let Some(static_tile) = self.static_tiles.get(&tile_pos) {
                Some(TileContent::Static(*static_tile))
            } else {
                Some(TileContent::Empty)
            }
        }
    }

    pub fn get_static_at(&self, tile_pos: TilePos) -> Option<StaticTile> {
        self.static_tiles.get(&tile_pos).copied()
    }

    pub fn get_entity_at(&self, tile_pos: TilePos) -> Option<Uuid> {
        self.entity_tiles.get(&tile_pos).copied()
    }

    pub fn set_static_tile(&mut self, pos: TilePos, tile: StaticTile) {
        self.static_tiles.insert(pos, tile);
        // TODO: Update spatial index
    }

    pub fn set_entity_tile(&mut self, pos: TilePos, entity_id: Uuid) {
        self.entity_tiles.insert(pos, entity_id);
        // TODO: Update spatial index
    }

    pub fn remove_tile(&mut self, pos: TilePos) {
        self.static_tiles.remove(&pos);
        self.entity_tiles.remove(&pos);
        // TODO: Update spatial index
    }

    // Convert world position to mech-local position if inside a mech
    fn world_to_mech_local(&self, _world_pos: WorldPos) -> Option<(Uuid, WorldPos)> {
        // TODO: Implement based on mech positions
        None
    }
}

// =============================================================================
// Mech Tile Map Implementation
// =============================================================================

impl MechTileMap {
    pub fn new(_mech_entity: Uuid, floor_count: usize) -> Self {
        let mut floors = Vec::with_capacity(floor_count);
        for _ in 0..floor_count {
            floors.push(FloorMap {
                static_tiles: HashMap::new(),
                entity_tiles: HashMap::new(),
            });
        }

        Self {
            floors,
            position: TilePos::new(0, 0), // Will be set when created
        }
    }

    pub fn get_floor(&self, floor_idx: usize) -> Option<&FloorMap> {
        self.floors.get(floor_idx)
    }

    pub fn get_floor_mut(&mut self, floor_idx: usize) -> Option<&mut FloorMap> {
        self.floors.get_mut(floor_idx)
    }
}

// =============================================================================
// Floor Map Implementation
// =============================================================================

impl FloorMap {
    pub fn set_static_tile(&mut self, pos: TilePos, tile: StaticTile) {
        self.static_tiles.insert(pos, tile);
    }

    pub fn set_entity_tile(&mut self, pos: TilePos, entity_id: Uuid) {
        self.entity_tiles.insert(pos, entity_id);
    }

    pub fn get_tile_at(&self, pos: TilePos) -> TileContent {
        if let Some(entity_id) = self.entity_tiles.get(&pos) {
            TileContent::Entity(*entity_id)
        } else if let Some(static_tile) = self.static_tiles.get(&pos) {
            TileContent::Static(*static_tile)
        } else {
            TileContent::Empty
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_tile_properties() {
        let floor = StaticTile::MetalFloor;
        assert!(floor.is_walkable());
        assert!(!floor.blocks_vision());

        let wall = StaticTile::MetalWall;
        assert!(!wall.is_walkable());
        assert!(wall.blocks_vision());

        let window = StaticTile::Window {
            facing: Direction::Up,
        };
        assert!(!window.is_walkable());
        assert!(!window.blocks_vision());
        assert_eq!(window.vision_attenuation(), 0.2);
    }

    #[test]
    fn test_tile_map_basic_operations() {
        let mut tile_map = TileMap::new();
        let pos = TilePos::new(5, 5);

        // Set a static tile
        tile_map.set_static_tile(pos, StaticTile::MetalFloor);
        assert_eq!(tile_map.get_static_at(pos), Some(StaticTile::MetalFloor));

        // Set an entity tile
        let entity_id = Uuid::new_v4();
        tile_map.set_entity_tile(pos, entity_id);
        assert_eq!(tile_map.get_entity_at(pos), Some(entity_id));

        // Remove tile
        tile_map.remove_tile(pos);
        assert_eq!(tile_map.get_static_at(pos), None);
        assert_eq!(tile_map.get_entity_at(pos), None);
    }
}

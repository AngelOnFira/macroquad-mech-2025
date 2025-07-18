use crate::{
    WorldTile, MechInteriorTile, TilePos, StationType, Direction,
    tile_entity::*, components::*
};
use uuid::Uuid;

// =============================================================================
// Migration Bridge
// =============================================================================
// This module provides utilities to convert between the old tile system
// (WorldTile, MechInteriorTile) and the new hybrid tile-entity system

pub struct TileMigration;

impl TileMigration {
    /// Convert old WorldTile to new TileContent
    pub fn world_tile_to_content(tile: &WorldTile) -> TileContent {
        match tile {
            WorldTile::Grass => TileContent::Empty, // Grass is the default, so we can use Empty
            WorldTile::Wall => TileContent::Static(StaticTile::MetalWall),
            WorldTile::Empty => TileContent::Empty,
            
            // These need special handling as they reference entities
            WorldTile::MechDoor { .. } | WorldTile::ResourceDropoff { .. } => {
                // For now, return Empty. These will be handled by entity creation
                TileContent::Empty
            }
        }
    }
    
    /// Convert old MechInteriorTile to new TileContent
    pub fn interior_tile_to_content(tile: &MechInteriorTile) -> TileContent {
        match tile {
            MechInteriorTile::Empty => TileContent::Empty,
            MechInteriorTile::Floor => TileContent::Static(StaticTile::MetalFloor),
            MechInteriorTile::Wall => TileContent::Static(StaticTile::MetalWall),
            MechInteriorTile::Ladder => TileContent::Static(StaticTile::TransitionZone {
                zone_id: 0,
                transition_type: TransitionType::Ladder,
            }),
            
            // These need special handling
            MechInteriorTile::Station(_) | MechInteriorTile::ExitDoor { .. } => {
                // For now, return Floor. Stations will be entities
                TileContent::Static(StaticTile::MetalFloor)
            }
        }
    }
    
    /// Create transition zones for mech doors
    pub fn create_door_transitions(_mech_id: Uuid, door_positions: &[(TilePos, Direction)]) -> Vec<(TilePos, StaticTile)> {
        let mut transitions = Vec::new();
        
        for (pos, facing) in door_positions {
            // First tile of transition
            transitions.push((
                *pos,
                StaticTile::TransitionZone {
                    zone_id: 0,
                    transition_type: TransitionType::MechEntrance { stage: 0 },
                }
            ));
            
            // Second tile of transition (in the facing direction)
            let (dx, dy) = facing.to_offset();
            let second_pos = TilePos::new(pos.x + dx, pos.y + dy);
            transitions.push((
                second_pos,
                StaticTile::TransitionZone {
                    zone_id: 0,
                    transition_type: TransitionType::MechEntrance { stage: 1 },
                }
            ));
        }
        
        transitions
    }
    
    /// Create stairs transitions between floors
    pub fn create_stair_transitions(positions: &[(TilePos, bool)]) -> Vec<(TilePos, StaticTile)> {
        let mut transitions = Vec::new();
        
        for (pos, going_up) in positions {
            let transition_type = if *going_up {
                TransitionType::StairUp { stage: 0 }
            } else {
                TransitionType::StairDown { stage: 0 }
            };
            
            // First tile
            transitions.push((
                *pos,
                StaticTile::TransitionZone {
                    zone_id: 1,
                    transition_type,
                }
            ));
            
            // Second tile (adjacent)
            let second_pos = TilePos::new(pos.x + 1, pos.y);
            let second_type = if *going_up {
                TransitionType::StairUp { stage: 1 }
            } else {
                TransitionType::StairDown { stage: 1 }
            };
            
            transitions.push((
                second_pos,
                StaticTile::TransitionZone {
                    zone_id: 1,
                    transition_type: second_type,
                }
            ));
        }
        
        transitions
    }
}

// =============================================================================
// Backward Compatibility Helpers
// =============================================================================

pub trait TileCompatibility {
    fn is_walkable_compat(&self) -> bool;
    fn blocks_projectiles_compat(&self) -> bool;
    fn to_old_world_tile(&self) -> Option<WorldTile>;
    fn to_old_interior_tile(&self) -> Option<MechInteriorTile>;
}

impl TileCompatibility for TileContent {
    fn is_walkable_compat(&self) -> bool {
        match self {
            TileContent::Empty => false,
            TileContent::Static(tile) => tile.is_walkable(),
            TileContent::Entity(_) => true, // Assume entities are walkable by default
        }
    }
    
    fn blocks_projectiles_compat(&self) -> bool {
        match self {
            TileContent::Empty => false,
            TileContent::Static(tile) => matches!(
                tile,
                StaticTile::MetalWall | StaticTile::ReinforcedWall
            ),
            TileContent::Entity(_) => false, // Need to check entity components
        }
    }
    
    fn to_old_world_tile(&self) -> Option<WorldTile> {
        match self {
            TileContent::Empty => Some(WorldTile::Grass),
            TileContent::Static(StaticTile::MetalWall) => Some(WorldTile::Wall),
            _ => None,
        }
    }
    
    fn to_old_interior_tile(&self) -> Option<MechInteriorTile> {
        match self {
            TileContent::Empty => Some(MechInteriorTile::Empty),
            TileContent::Static(StaticTile::MetalFloor) => Some(MechInteriorTile::Floor),
            TileContent::Static(StaticTile::MetalWall) => Some(MechInteriorTile::Wall),
            TileContent::Static(StaticTile::TransitionZone { 
                transition_type: TransitionType::Ladder, .. 
            }) => Some(MechInteriorTile::Ladder),
            _ => None,
        }
    }
}

// =============================================================================
// Entity Templates for Stations
// =============================================================================

pub fn create_station_template(station_type: StationType) -> EntityTemplate {
    let base_components = EntityComponents {
        station: Some(Station {
            station_type,
            interaction_range: 1.5,
            power_required: match station_type {
                StationType::WeaponLaser => 100.0,
                StationType::WeaponProjectile => 80.0,
                StationType::Shield => 150.0,
                StationType::Engine => 50.0,
                StationType::Repair => 30.0,
                StationType::Electrical => 40.0,
                StationType::Upgrade => 20.0,
                StationType::Pilot => 60.0,
            },
            operating: false,
        }),
        solid: Some(Solid {
            blocks_movement: false,
            blocks_projectiles: false,
        }),
        interactable: Some(Interactable {
            prompt: format!("Press E to operate {:?}", station_type),
            range: 1.5,
            requires_facing: false,
        }),
        renderable: Some(Renderable {
            sprite: SpriteId(station_type as u32),
            layer: RenderLayer::Object,
            color_modulation: Color::WHITE,
            animation_state: None,
        }),
        ..Default::default()
    };
    
    EntityTemplate {
        name: format!("{:?} Station", station_type),
        components: base_components,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_world_tile_migration() {
        assert!(matches!(
            TileMigration::world_tile_to_content(&WorldTile::Grass),
            TileContent::Empty
        ));
        
        assert!(matches!(
            TileMigration::world_tile_to_content(&WorldTile::Wall),
            TileContent::Static(StaticTile::MetalWall)
        ));
    }
    
    #[test]
    fn test_interior_tile_migration() {
        assert!(matches!(
            TileMigration::interior_tile_to_content(&MechInteriorTile::Floor),
            TileContent::Static(StaticTile::MetalFloor)
        ));
        
        assert!(matches!(
            TileMigration::interior_tile_to_content(&MechInteriorTile::Wall),
            TileContent::Static(StaticTile::MetalWall)
        ));
    }
    
    #[test]
    fn test_tile_compatibility() {
        let floor = TileContent::Static(StaticTile::MetalFloor);
        assert!(floor.is_walkable_compat());
        assert!(!floor.blocks_projectiles_compat());
        
        let wall = TileContent::Static(StaticTile::MetalWall);
        assert!(!wall.is_walkable_compat());
        assert!(wall.blocks_projectiles_compat());
    }
}
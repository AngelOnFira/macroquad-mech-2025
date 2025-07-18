use std::collections::{HashMap, HashSet};
use crate::{WorldPos, TilePos, Direction, tile_entity::*, components::*};

// =============================================================================
// Vision System
// =============================================================================

pub struct VisionSystem {
    // Cached visibility data per viewer
    visibility_cache: HashMap<uuid::Uuid, VisibilityData>,
}

#[derive(Debug, Clone)]
pub struct VisibilityData {
    pub visible_tiles: HashSet<TilePos>,
    pub light_levels: HashMap<TilePos, f32>,
    pub last_update_pos: WorldPos,
}

pub struct Ray {
    _origin: WorldPos,
    angle: f32,
    current_pos: WorldPos,
    length: f32,
}

impl Ray {
    pub fn new(origin: WorldPos, angle: f32) -> Self {
        Self {
            _origin: origin,
            angle,
            current_pos: origin,
            length: 0.0,
        }
    }
    
    pub fn current_pos(&self) -> WorldPos {
        self.current_pos
    }
    
    pub fn advance(&mut self, step: f32) {
        let (dx, dy) = angle_to_direction(self.angle);
        self.current_pos.x += dx * step;
        self.current_pos.y += dy * step;
        self.length += step;
    }
}

impl VisionSystem {
    pub fn new() -> Self {
        Self {
            visibility_cache: HashMap::new(),
        }
    }
    
    pub fn calculate_visibility<S: ComponentStorage>(
        &mut self,
        viewer_id: uuid::Uuid,
        viewer_pos: WorldPos,
        max_range: f32,
        tile_map: &TileMap,
        component_storage: &S,
    ) -> &VisibilityData {
        // Check if we need to recalculate
        let needs_update = if let Some(cached) = self.visibility_cache.get(&viewer_id) {
            let dx = cached.last_update_pos.x - viewer_pos.x;
            let dy = cached.last_update_pos.y - viewer_pos.y;
            dx.abs() >= 0.1 || dy.abs() >= 0.1
        } else {
            true
        };
        
        if !needs_update {
            return self.visibility_cache.get(&viewer_id).unwrap();
        }
        
        let mut visible = HashSet::new();
        let mut light_levels = HashMap::new();
        
        // Cast rays in all directions
        for angle in 0..360 {
            let mut ray = Ray::new(viewer_pos, angle as f32);
            let mut attenuation = 0.0;
            
            while ray.length < max_range && attenuation < 1.0 {
                let check_pos = ray.current_pos();
                let tile_pos = check_pos.to_tile();
                
                if let Some(tile_content) = tile_map.get_tile_at(check_pos) {
                    match tile_content {
                        TileContent::Static(static_tile) => {
                            attenuation += static_tile.vision_attenuation();
                            if static_tile.blocks_vision() {
                                break;
                            }
                        }
                        TileContent::Entity(entity_id) => {
                            if let Some(opaque) = component_storage.get_opaque(entity_id) {
                                attenuation += opaque.attenuation;
                                if opaque.blocks_completely {
                                    break;
                                }
                            }
                        }
                        TileContent::Empty => {
                            // Empty tiles don't affect vision
                        }
                    }
                }
                
                if attenuation < 1.0 {
                    visible.insert(tile_pos);
                    light_levels.insert(tile_pos, 1.0 - attenuation);
                }
                
                ray.advance(0.5);
            }
        }
        
        // Cache and return
        let visibility_data = VisibilityData {
            visible_tiles: visible,
            light_levels,
            last_update_pos: viewer_pos,
        };
        
        self.visibility_cache.insert(viewer_id, visibility_data);
        self.visibility_cache.get(&viewer_id).unwrap()
    }
    
    pub fn get_visibility(&self, viewer_id: uuid::Uuid) -> Option<&VisibilityData> {
        self.visibility_cache.get(&viewer_id)
    }
    
    pub fn clear_cache(&mut self) {
        self.visibility_cache.clear();
    }
    
    pub fn remove_viewer(&mut self, viewer_id: uuid::Uuid) {
        self.visibility_cache.remove(&viewer_id);
    }
}

// =============================================================================
// Enhanced Vision for Windows
// =============================================================================

pub struct WindowVision {
    pub base_radius: f32,
    pub window_extension: f32,
}

impl WindowVision {
    pub fn new(base_radius: f32, window_extension: f32) -> Self {
        Self {
            base_radius,
            window_extension,
        }
    }
    
    pub fn calculate_window_visibility<S: ComponentStorage>(
        &self,
        viewer_pos: WorldPos,
        viewer_inside_mech: bool,
        tile_map: &TileMap,
        _component_storage: &S,
    ) -> VisibilityResult {
        let mut result = VisibilityResult {
            visible_tiles: HashSet::new(),
            window_views: Vec::new(),
        };
        
        if !viewer_inside_mech {
            // Outside viewers use normal vision
            return result;
        }
        
        // Find nearby windows
        let search_radius = self.base_radius;
        let viewer_tile = viewer_pos.to_tile();
        
        for dx in -search_radius as i32..=search_radius as i32 {
            for dy in -search_radius as i32..=search_radius as i32 {
                let check_tile = TilePos::new(viewer_tile.x + dx, viewer_tile.y + dy);
                let check_pos = check_tile.to_world();
                
                if let Some(TileContent::Static(static_tile)) = tile_map.get_tile_at(check_pos) {
                    match static_tile {
                        StaticTile::Window { facing } | 
                        StaticTile::ReinforcedWindow { facing, .. } => {
                            // Calculate window view cone
                            let window_view = self.calculate_window_cone(
                                check_tile,
                                facing,
                                viewer_pos,
                                self.window_extension,
                            );
                            result.window_views.push(window_view);
                        }
                        _ => {}
                    }
                }
            }
        }
        
        result
    }
    
    fn calculate_window_cone(
        &self,
        window_pos: TilePos,
        facing: Direction,
        viewer_pos: WorldPos,
        extension: f32,
    ) -> WindowView {
        let window_world = window_pos.to_world();
        
        // Calculate angle from viewer to window
        let dx = window_world.x - viewer_pos.x;
        let dy = window_world.y - viewer_pos.y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // Window provides vision in its facing direction
        let base_angle = direction_to_angle(facing);
        let cone_width = 60.0; // 60 degree cone
        
        WindowView {
            window_pos,
            facing,
            viewer_distance: distance,
            vision_cone: VisionCone {
                origin: window_world,
                direction: base_angle,
                width: cone_width,
                range: extension,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct VisibilityResult {
    pub visible_tiles: HashSet<TilePos>,
    pub window_views: Vec<WindowView>,
}

#[derive(Debug, Clone)]
pub struct WindowView {
    pub window_pos: TilePos,
    pub facing: Direction,
    pub viewer_distance: f32,
    pub vision_cone: VisionCone,
}

#[derive(Debug, Clone)]
pub struct VisionCone {
    pub origin: WorldPos,
    pub direction: f32, // Angle in degrees
    pub width: f32,     // Cone width in degrees
    pub range: f32,     // How far the cone extends
}

// =============================================================================
// Helper Functions
// =============================================================================

fn angle_to_direction(angle: f32) -> (f32, f32) {
    let radians = angle.to_radians();
    (radians.cos(), radians.sin())
}

fn direction_to_angle(direction: Direction) -> f32 {
    match direction {
        Direction::Up => 270.0,
        Direction::Down => 90.0,
        Direction::Left => 180.0,
        Direction::Right => 0.0,
    }
}

// =============================================================================
// Movement Error Types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementError {
    Blocked,
    OutOfBounds,
    InvalidDestination,
}

// =============================================================================
// Movement System Integration
// =============================================================================

pub fn handle_movement<S: ComponentStorage>(
    tile_map: &TileMap,
    component_storage: &S,
    _entity: uuid::Uuid,
    new_pos: WorldPos,
) -> Result<(), MovementError> {
    // Check static tile at destination
    let _tile_pos = new_pos.to_tile();
    
    if let Some(tile_content) = tile_map.get_tile_at(new_pos) {
        match tile_content {
            TileContent::Static(static_tile) => {
                if !static_tile.is_walkable() {
                    return Err(MovementError::Blocked);
                }
            }
            TileContent::Entity(entity_id) => {
                // Check if entity blocks movement
                if let Some(solid) = component_storage.get_solid(entity_id) {
                    if solid.blocks_movement {
                        return Err(MovementError::Blocked);
                    }
                }
            }
            TileContent::Empty => {
                // Empty tiles might be out of bounds
                return Err(MovementError::OutOfBounds);
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[test]
    fn test_vision_system_creation() {
        let mut vision_system = VisionSystem::new();
        let viewer_id = Uuid::new_v4();
        
        assert!(vision_system.get_visibility(viewer_id).is_none());
        
        vision_system.visibility_cache.insert(viewer_id, VisibilityData {
            visible_tiles: HashSet::new(),
            light_levels: HashMap::new(),
            last_update_pos: WorldPos::new(0.0, 0.0),
        });
        
        assert!(vision_system.get_visibility(viewer_id).is_some());
    }
    
    #[test]
    fn test_ray_advancement() {
        let mut ray = Ray::new(WorldPos::new(0.0, 0.0), 0.0);
        
        ray.advance(10.0);
        assert_eq!(ray.current_pos().x, 10.0);
        assert_eq!(ray.current_pos().y, 0.0);
        assert_eq!(ray.length, 10.0);
    }
    
    #[test]
    fn test_angle_conversions() {
        let (dx, dy) = angle_to_direction(0.0);
        assert!((dx - 1.0).abs() < 0.001);
        assert!(dy.abs() < 0.001);
        
        let (dx, dy) = angle_to_direction(90.0);
        assert!(dx.abs() < 0.001);
        assert!((dy - 1.0).abs() < 0.001);
    }
}
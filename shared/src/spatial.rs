use std::collections::HashMap;
use uuid::Uuid;
use crate::{WorldPos, TILE_SIZE, ARENA_WIDTH_TILES, ARENA_HEIGHT_TILES};
#[cfg(not(target_arch = "wasm32"))]
use crate::uuid_gen::new_uuid;

/// Spatial partitioning system for efficient collision detection
pub struct SpatialGrid<T> {
    grid: HashMap<GridCell, Vec<SpatialEntity<T>>>,
    cell_size: f32,
    width: i32,
    height: i32,
}

/// A cell in the spatial grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: i32,
    pub y: i32,
}

/// An entity stored in the spatial grid
#[derive(Debug, Clone)]
pub struct SpatialEntity<T> {
    pub id: Uuid,
    pub position: WorldPos,
    pub radius: f32,
    pub data: T,
}

/// Spatial query result
#[derive(Debug, Clone)]
pub struct SpatialQueryResult<T> {
    pub entity: SpatialEntity<T>,
    pub distance: f32,
}

impl<T: Clone> SpatialGrid<T> {
    /// Create a new spatial grid
    pub fn new(cell_size: f32, world_width: f32, world_height: f32) -> Self {
        let width = (world_width / cell_size).ceil() as i32;
        let height = (world_height / cell_size).ceil() as i32;
        
        Self {
            grid: HashMap::new(),
            cell_size,
            width,
            height,
        }
    }
    
    /// Create a spatial grid sized for the game arena
    pub fn for_arena(cell_size: f32) -> Self {
        let world_width = ARENA_WIDTH_TILES as f32 * TILE_SIZE;
        let world_height = ARENA_HEIGHT_TILES as f32 * TILE_SIZE;
        Self::new(cell_size, world_width, world_height)
    }
    
    /// Clear all entities from the grid
    pub fn clear(&mut self) {
        self.grid.clear();
    }
    
    /// Insert an entity into the grid
    pub fn insert(&mut self, entity: SpatialEntity<T>) {
        let cells = self.get_cells_for_entity(&entity);
        for cell in cells {
            self.grid.entry(cell).or_insert_with(Vec::new).push(entity.clone());
        }
    }
    
    /// Remove an entity from the grid
    pub fn remove(&mut self, entity_id: Uuid) {
        for entities in self.grid.values_mut() {
            entities.retain(|e| e.id != entity_id);
        }
        // Clean up empty cells
        self.grid.retain(|_, entities| !entities.is_empty());
    }
    
    /// Update an entity's position in the grid
    pub fn update(&mut self, entity: SpatialEntity<T>) {
        self.remove(entity.id);
        self.insert(entity);
    }
    
    /// Query entities within a radius of a position
    pub fn query_radius(&self, center: WorldPos, radius: f32) -> Vec<SpatialQueryResult<T>> {
        let mut results = Vec::new();
        let cells = self.get_cells_for_circle(center, radius);
        
        for cell in cells {
            if let Some(entities) = self.grid.get(&cell) {
                for entity in entities {
                    let distance = center.distance_to(entity.position);
                    if distance <= radius + entity.radius {
                        results.push(SpatialQueryResult {
                            entity: entity.clone(),
                            distance,
                        });
                    }
                }
            }
        }
        
        // Remove duplicates and sort by distance
        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        results.dedup_by(|a, b| a.entity.id == b.entity.id);
        results
    }
    
    /// Query entities within a rectangular area
    pub fn query_rect(&self, min: WorldPos, max: WorldPos) -> Vec<SpatialEntity<T>> {
        let mut results = Vec::new();
        let cells = self.get_cells_for_rect(min, max);
        
        for cell in cells {
            if let Some(entities) = self.grid.get(&cell) {
                for entity in entities {
                    if entity.position.x >= min.x && entity.position.x <= max.x &&
                       entity.position.y >= min.y && entity.position.y <= max.y {
                        results.push(entity.clone());
                    }
                }
            }
        }
        
        // Remove duplicates
        results.sort_by(|a, b| a.id.cmp(&b.id));
        results.dedup_by(|a, b| a.id == b.id);
        results
    }
    
    /// Get the nearest entity to a position
    pub fn get_nearest(&self, position: WorldPos, max_distance: f32) -> Option<SpatialQueryResult<T>> {
        self.query_radius(position, max_distance)
            .into_iter()
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
    }
    
    /// Get all entities in the grid
    pub fn get_all(&self) -> Vec<SpatialEntity<T>> {
        let mut results = Vec::new();
        for entities in self.grid.values() {
            results.extend(entities.clone());
        }
        
        // Remove duplicates
        results.sort_by(|a, b| a.id.cmp(&b.id));
        results.dedup_by(|a, b| a.id == b.id);
        results
    }
    
    /// Get debug information about the grid
    pub fn debug_info(&self) -> SpatialGridDebugInfo {
        let total_entities = self.get_all().len();
        let occupied_cells = self.grid.len();
        let total_cells = (self.width * self.height) as usize;
        
        SpatialGridDebugInfo {
            total_entities,
            occupied_cells,
            total_cells,
            cell_size: self.cell_size,
            width: self.width,
            height: self.height,
        }
    }
    
    /// Convert world position to grid cell
    fn world_to_cell(&self, pos: WorldPos) -> GridCell {
        let x = (pos.x / self.cell_size).floor() as i32;
        let y = (pos.y / self.cell_size).floor() as i32;
        GridCell {
            x: x.max(0).min(self.width - 1),
            y: y.max(0).min(self.height - 1),
        }
    }
    
    /// Get all cells that an entity occupies
    fn get_cells_for_entity(&self, entity: &SpatialEntity<T>) -> Vec<GridCell> {
        let center_cell = self.world_to_cell(entity.position);
        let radius_in_cells = (entity.radius / self.cell_size).ceil() as i32;
        
        let mut cells = Vec::new();
        for dx in -radius_in_cells..=radius_in_cells {
            for dy in -radius_in_cells..=radius_in_cells {
                let x = center_cell.x + dx;
                let y = center_cell.y + dy;
                if x >= 0 && x < self.width && y >= 0 && y < self.height {
                    cells.push(GridCell { x, y });
                }
            }
        }
        cells
    }
    
    /// Get all cells within a circular query
    fn get_cells_for_circle(&self, center: WorldPos, radius: f32) -> Vec<GridCell> {
        let center_cell = self.world_to_cell(center);
        let radius_in_cells = (radius / self.cell_size).ceil() as i32;
        
        let mut cells = Vec::new();
        for dx in -radius_in_cells..=radius_in_cells {
            for dy in -radius_in_cells..=radius_in_cells {
                let x = center_cell.x + dx;
                let y = center_cell.y + dy;
                if x >= 0 && x < self.width && y >= 0 && y < self.height {
                    cells.push(GridCell { x, y });
                }
            }
        }
        cells
    }
    
    /// Get all cells within a rectangular query
    fn get_cells_for_rect(&self, min: WorldPos, max: WorldPos) -> Vec<GridCell> {
        let min_cell = self.world_to_cell(min);
        let max_cell = self.world_to_cell(max);
        
        let mut cells = Vec::new();
        for x in min_cell.x..=max_cell.x {
            for y in min_cell.y..=max_cell.y {
                if x >= 0 && x < self.width && y >= 0 && y < self.height {
                    cells.push(GridCell { x, y });
                }
            }
        }
        cells
    }
}

/// Debug information about the spatial grid
#[derive(Debug, Clone)]
pub struct SpatialGridDebugInfo {
    pub total_entities: usize,
    pub occupied_cells: usize,
    pub total_cells: usize,
    pub cell_size: f32,
    pub width: i32,
    pub height: i32,
}

impl<T> SpatialEntity<T> {
    /// Create a new spatial entity
    pub fn new(id: Uuid, position: WorldPos, radius: f32, data: T) -> Self {
        Self {
            id,
            position,
            radius,
            data,
        }
    }
    
    /// Check if this entity collides with another
    pub fn collides_with(&self, other: &SpatialEntity<T>) -> bool {
        let distance = self.position.distance_to(other.position);
        distance <= self.radius + other.radius
    }
    
    /// Check if this entity collides with a point
    pub fn collides_with_point(&self, point: WorldPos) -> bool {
        let distance = self.position.distance_to(point);
        distance <= self.radius
    }
}

/// Utility functions for common spatial operations
pub mod spatial_utils {
    use super::*;
    
    /// Check if two circles overlap
    pub fn circles_overlap(pos1: WorldPos, radius1: f32, pos2: WorldPos, radius2: f32) -> bool {
        let distance = pos1.distance_to(pos2);
        distance <= radius1 + radius2
    }
    
    /// Check if a point is inside a rectangle
    pub fn point_in_rect(point: WorldPos, min: WorldPos, max: WorldPos) -> bool {
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }
    
    /// Get the closest point on a rectangle to a given point
    pub fn closest_point_on_rect(point: WorldPos, min: WorldPos, max: WorldPos) -> WorldPos {
        WorldPos::new(
            point.x.max(min.x).min(max.x),
            point.y.max(min.y).min(max.y),
        )
    }
    
    /// Check if a circle intersects with a rectangle
    pub fn circle_rect_intersection(
        circle_pos: WorldPos,
        circle_radius: f32,
        rect_min: WorldPos,
        rect_max: WorldPos,
    ) -> bool {
        let closest_point = closest_point_on_rect(circle_pos, rect_min, rect_max);
        let distance = circle_pos.distance_to(closest_point);
        distance <= circle_radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_grid_creation() {
        let grid: SpatialGrid<String> = SpatialGrid::new(100.0, 1000.0, 1000.0);
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert_eq!(grid.cell_size, 100.0);
    }
    
    #[test]
    fn test_entity_insertion_and_query() {
        let mut grid: SpatialGrid<String> = SpatialGrid::new(100.0, 1000.0, 1000.0);
        
        let entity = SpatialEntity::new(
            new_uuid(),
            WorldPos::new(150.0, 150.0),
            10.0,
            "test".to_string(),
        );
        
        grid.insert(entity.clone());
        
        let results = grid.query_radius(WorldPos::new(150.0, 150.0), 50.0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entity.id, entity.id);
    }
    
    #[test]
    fn test_entity_removal() {
        let mut grid: SpatialGrid<String> = SpatialGrid::new(100.0, 1000.0, 1000.0);
        
        let entity = SpatialEntity::new(
            new_uuid(),
            WorldPos::new(150.0, 150.0),
            10.0,
            "test".to_string(),
        );
        
        grid.insert(entity.clone());
        grid.remove(entity.id);
        
        let results = grid.query_radius(WorldPos::new(150.0, 150.0), 50.0);
        assert_eq!(results.len(), 0);
    }
    
    #[test]
    fn test_spatial_utils() {
        use spatial_utils::*;
        
        let pos1 = WorldPos::new(0.0, 0.0);
        let pos2 = WorldPos::new(10.0, 0.0);
        
        assert!(circles_overlap(pos1, 6.0, pos2, 6.0));
        assert!(!circles_overlap(pos1, 4.0, pos2, 4.0));
        
        let point = WorldPos::new(5.0, 5.0);
        let min = WorldPos::new(0.0, 0.0);
        let max = WorldPos::new(10.0, 10.0);
        
        assert!(point_in_rect(point, min, max));
        assert!(!point_in_rect(WorldPos::new(15.0, 5.0), min, max));
    }
}
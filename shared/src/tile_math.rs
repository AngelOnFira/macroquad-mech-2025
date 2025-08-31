use crate::{TILE_SIZE, MECH_SIZE_TILES};
use crate::coordinates::{WorldPos, TilePos, TileRegion};
use std::f32;

/// Tile alignment utilities
pub struct TileAlign;

impl TileAlign {
    /// Snap a world position to the center of its tile
    pub fn snap_to_tile_center(world_pos: WorldPos) -> WorldPos {
        let tile_pos = world_pos.to_tile();
        tile_pos.to_world_center()
    }
    
    /// Snap a world position to the nearest tile corner
    pub fn snap_to_tile_corner(world_pos: WorldPos) -> WorldPos {
        let tile_pos = world_pos.to_tile();
        tile_pos.to_world()
    }
    
    /// Snap a world position to the nearest grid intersection
    pub fn snap_to_grid(world_pos: WorldPos, grid_size: f32) -> WorldPos {
        WorldPos::new(
            (world_pos.x / grid_size).round() * grid_size,
            (world_pos.y / grid_size).round() * grid_size,
        )
    }
    
    /// Check if a world position is aligned to tile boundaries
    pub fn is_tile_aligned(world_pos: WorldPos) -> bool {
        let tile_pos = world_pos.to_tile();
        let aligned_pos = tile_pos.to_world();
        (world_pos.x - aligned_pos.x).abs() < f32::EPSILON &&
        (world_pos.y - aligned_pos.y).abs() < f32::EPSILON
    }
}

/// Distance calculations optimized for tile-based games
pub struct TileDistance;

impl TileDistance {
    /// Calculate distance in "tile units" (i.e., how many tiles apart)
    pub fn tile_distance(pos1: WorldPos, pos2: WorldPos) -> f32 {
        let dx = (pos1.x - pos2.x) / TILE_SIZE;
        let dy = (pos1.y - pos2.y) / TILE_SIZE;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// Calculate Manhattan distance in tile units
    pub fn tile_manhattan_distance(pos1: WorldPos, pos2: WorldPos) -> f32 {
        let dx = ((pos1.x - pos2.x) / TILE_SIZE).abs();
        let dy = ((pos1.y - pos2.y) / TILE_SIZE).abs();
        dx + dy
    }
    
    /// Check if two positions are within a certain tile radius
    pub fn within_tile_radius(pos1: WorldPos, pos2: WorldPos, radius: f32) -> bool {
        Self::tile_distance(pos1, pos2) <= radius
    }
    
    /// Get the closest point on a tile to a given world position
    pub fn closest_point_on_tile(world_pos: WorldPos, tile_pos: TilePos) -> WorldPos {
        let tile_world = tile_pos.to_world();
        let tile_max_x = tile_world.x + TILE_SIZE;
        let tile_max_y = tile_world.y + TILE_SIZE;
        
        WorldPos::new(
            world_pos.x.max(tile_world.x).min(tile_max_x),
            world_pos.y.max(tile_world.y).min(tile_max_y),
        )
    }
}

/// Pathfinding and navigation utilities for tile-based movement
pub struct TileNavigation;

impl TileNavigation {
    /// Get all valid adjacent tiles (4-directional)
    pub fn adjacent_tiles(center: TilePos, bounds: Option<TileRegion>) -> Vec<TilePos> {
        let neighbors = center.neighbors_4();
        let bounds = bounds.unwrap_or(TileRegion::world_bounds());
        
        neighbors
            .into_iter()
            .filter(|&pos| bounds.contains(pos))
            .collect()
    }
    
    /// Get all valid adjacent tiles (8-directional)
    pub fn adjacent_tiles_8dir(center: TilePos, bounds: Option<TileRegion>) -> Vec<TilePos> {
        let neighbors = center.neighbors_8();
        let bounds = bounds.unwrap_or(TileRegion::world_bounds());
        
        neighbors
            .into_iter()
            .filter(|&pos| bounds.contains(pos))
            .collect()
    }
    
    /// Calculate the tiles along a straight line between two points (Bresenham's algorithm)
    pub fn line_of_tiles(start: TilePos, end: TilePos) -> Vec<TilePos> {
        let mut tiles = Vec::new();
        
        let dx = (end.x - start.x).abs();
        let dy = (end.y - start.y).abs();
        let sx = if start.x < end.x { 1 } else { -1 };
        let sy = if start.y < end.y { 1 } else { -1 };
        let mut err = dx - dy;
        
        let mut current = start;
        loop {
            tiles.push(current);
            
            if current == end {
                break;
            }
            
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                current.x += sx;
            }
            if e2 < dx {
                err += dx;
                current.y += sy;
            }
        }
        
        tiles
    }
    
    /// Get all tiles within a circular radius
    pub fn tiles_in_circle(center: TilePos, radius: f32) -> Vec<TilePos> {
        let mut tiles = Vec::new();
        let bounds = TileRegion::from_center_radius(center, radius.ceil() as i32);
        
        for tile_pos in bounds.iter() {
            let distance = center.distance_to(tile_pos);
            if distance <= radius {
                tiles.push(tile_pos);
            }
        }
        
        tiles
    }
    
    /// Get all tiles within a rectangular area
    pub fn tiles_in_rectangle(center: TilePos, width: i32, height: i32) -> Vec<TilePos> {
        let half_width = width / 2;
        let half_height = height / 2;
        let region = TileRegion::new(
            TilePos::new(center.x - half_width, center.y - half_height),
            TilePos::new(center.x + half_width, center.y + half_height),
        );
        
        region.iter().collect()
    }
}

/// Specialized calculations for mech-related positioning
pub struct MechPositioning;

impl MechPositioning {
    /// Calculate the world bounds of a mech at a given position
    pub fn mech_world_bounds(mech_tile_pos: TilePos) -> (WorldPos, WorldPos) {
        let min = mech_tile_pos.to_world();
        let max = WorldPos::new(
            min.x + MECH_SIZE_TILES as f32 * TILE_SIZE,
            min.y + MECH_SIZE_TILES as f32 * TILE_SIZE,
        );
        (min, max)
    }
    
    /// Check if a world position is inside a mech
    pub fn is_inside_mech(world_pos: WorldPos, mech_tile_pos: TilePos) -> bool {
        let (min, max) = Self::mech_world_bounds(mech_tile_pos);
        world_pos.x >= min.x && world_pos.x < max.x &&
        world_pos.y >= min.y && world_pos.y < max.y
    }
    
    /// Get the center position of a mech in world coordinates
    pub fn mech_center(mech_tile_pos: TilePos) -> WorldPos {
        let mech_world = mech_tile_pos.to_world();
        WorldPos::new(
            mech_world.x + (MECH_SIZE_TILES as f32 * TILE_SIZE) / 2.0,
            mech_world.y + (MECH_SIZE_TILES as f32 * TILE_SIZE) / 2.0,
        )
    }
    
    /// Calculate valid spawn positions around a mech (outside its bounds)
    pub fn spawn_positions_around_mech(mech_tile_pos: TilePos, min_distance: i32) -> Vec<WorldPos> {
        let mut positions = Vec::new();
        let mech_region = TileRegion::new(
            mech_tile_pos,
            TilePos::new(
                mech_tile_pos.x + MECH_SIZE_TILES - 1,
                mech_tile_pos.y + MECH_SIZE_TILES - 1,
            ),
        );
        
        // Generate positions around the mech perimeter
        let expanded_region = TileRegion::new(
            TilePos::new(
                mech_region.min.x - min_distance,
                mech_region.min.y - min_distance,
            ),
            TilePos::new(
                mech_region.max.x + min_distance,
                mech_region.max.y + min_distance,
            ),
        );
        
        for tile_pos in expanded_region.iter() {
            // Skip tiles that are inside or too close to the mech
            if !mech_region.contains(tile_pos) {
                // Check if it's at least min_distance away from mech edges
                let closest_mech_tile = mech_region.clamp(tile_pos);
                if tile_pos.manhattan_distance_to(closest_mech_tile) >= min_distance {
                    positions.push(tile_pos.to_world_center());
                }
            }
        }
        
        positions
    }
}

/// Utilities for working with areas and regions
pub struct AreaCalculations;

impl AreaCalculations {
    /// Calculate the overlapping area between two rectangular regions in world coordinates
    pub fn overlap_area(
        pos1: WorldPos, size1: (f32, f32),
        pos2: WorldPos, size2: (f32, f32),
    ) -> f32 {
        let left = f32::max(pos1.x, pos2.x);
        let right = f32::min(pos1.x + size1.0, pos2.x + size2.0);
        let top = f32::max(pos1.y, pos2.y);
        let bottom = f32::min(pos1.y + size1.1, pos2.y + size2.1);
        
        if left < right && top < bottom {
            (right - left) * (bottom - top)
        } else {
            0.0
        }
    }
    
    /// Check if two rectangular regions overlap
    pub fn regions_overlap(
        pos1: WorldPos, size1: (f32, f32),
        pos2: WorldPos, size2: (f32, f32),
    ) -> bool {
        Self::overlap_area(pos1, size1, pos2, size2) > 0.0
    }
    
    /// Calculate the area of a tile region in world units
    pub fn tile_region_world_area(region: &TileRegion) -> f32 {
        (region.width() as f32 * TILE_SIZE) * (region.height() as f32 * TILE_SIZE)
    }
    
    /// Get all tiles that intersect with a circular area in world space
    pub fn tiles_intersecting_circle(center: WorldPos, radius: f32) -> Vec<TilePos> {
        let center_tile = center.to_tile();
        let tile_radius = (radius / TILE_SIZE).ceil() as i32;
        let search_region = TileRegion::from_center_radius(center_tile, tile_radius);
        
        let mut intersecting_tiles = Vec::new();
        
        for tile_pos in search_region.iter() {
            // Check if circle intersects with tile bounds
            let tile_world = tile_pos.to_world();
            let closest_point = WorldPos::new(
                center.x.max(tile_world.x).min(tile_world.x + TILE_SIZE),
                center.y.max(tile_world.y).min(tile_world.y + TILE_SIZE),
            );
            
            if center.distance_to(closest_point) <= radius {
                intersecting_tiles.push(tile_pos);
            }
        }
        
        intersecting_tiles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_align() {
        let world_pos = WorldPos::new(35.7, 67.3);
        let snapped = TileAlign::snap_to_tile_center(world_pos);
        
        // Should snap to center of tile (1, 2) assuming TILE_SIZE = 32
        let expected = WorldPos::new(1.0 * TILE_SIZE + TILE_SIZE / 2.0, 2.0 * TILE_SIZE + TILE_SIZE / 2.0);
        assert_eq!(snapped, expected);
    }

    #[test]
    fn test_tile_distance() {
        let pos1 = WorldPos::new(0.0, 0.0);
        let pos2 = WorldPos::new(TILE_SIZE * 3.0, TILE_SIZE * 4.0);
        
        let distance = TileDistance::tile_distance(pos1, pos2);
        assert_eq!(distance, 5.0); // 3-4-5 triangle
        
        let manhattan = TileDistance::tile_manhattan_distance(pos1, pos2);
        assert_eq!(manhattan, 7.0);
    }

    #[test]
    fn test_line_of_tiles() {
        let start = TilePos::new(0, 0);
        let end = TilePos::new(2, 1);
        let line = TileNavigation::line_of_tiles(start, end);
        
        assert!(line.contains(&start));
        assert!(line.contains(&end));
        assert!(line.len() >= 3); // At least start, middle, end
    }

    #[test]
    fn test_tiles_in_circle() {
        let center = TilePos::new(5, 5);
        let tiles = TileNavigation::tiles_in_circle(center, 1.5);
        
        // Should include center and immediate neighbors
        assert!(tiles.contains(&center));
        assert!(tiles.contains(&TilePos::new(4, 5))); // Left neighbor
        assert!(tiles.contains(&TilePos::new(6, 5))); // Right neighbor
        assert!(tiles.len() > 1);
    }

    #[test]
    fn test_mech_positioning() {
        let mech_pos = TilePos::new(10, 10);
        let center = MechPositioning::mech_center(mech_pos);
        
        let expected_x = 10.0 * TILE_SIZE + (MECH_SIZE_TILES as f32 * TILE_SIZE) / 2.0;
        let expected_y = 10.0 * TILE_SIZE + (MECH_SIZE_TILES as f32 * TILE_SIZE) / 2.0;
        
        assert_eq!(center.x, expected_x);
        assert_eq!(center.y, expected_y);
        
        // Test point inside mech
        let inside_point = WorldPos::new(expected_x, expected_y);
        assert!(MechPositioning::is_inside_mech(inside_point, mech_pos));
        
        // Test point outside mech
        let outside_point = WorldPos::new(0.0, 0.0);
        assert!(!MechPositioning::is_inside_mech(outside_point, mech_pos));
    }

    #[test]
    fn test_area_calculations() {
        let pos1 = WorldPos::new(0.0, 0.0);
        let size1 = (100.0, 100.0);
        let pos2 = WorldPos::new(50.0, 50.0);
        let size2 = (100.0, 100.0);
        
        let overlap = AreaCalculations::overlap_area(pos1, size1, pos2, size2);
        assert_eq!(overlap, 50.0 * 50.0); // 50x50 overlap
        
        assert!(AreaCalculations::regions_overlap(pos1, size1, pos2, size2));
        
        let pos3 = WorldPos::new(200.0, 200.0);
        assert!(!AreaCalculations::regions_overlap(pos1, size1, pos3, size2));
    }

    #[test]
    fn test_tiles_intersecting_circle() {
        let center = WorldPos::new(TILE_SIZE * 2.5, TILE_SIZE * 2.5); // Center of 4 tiles
        let tiles = AreaCalculations::tiles_intersecting_circle(center, TILE_SIZE * 0.8);
        
        // Should intersect with the 4 tiles around the center point
        assert!(tiles.len() >= 4);
    }
}
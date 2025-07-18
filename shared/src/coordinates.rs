use crate::{TILE_SIZE, ARENA_WIDTH_TILES, ARENA_HEIGHT_TILES, FLOOR_WIDTH_TILES, FLOOR_HEIGHT_TILES};
use std::ops::{Add, Sub, Mul, Div};

/// A unified coordinate system for the game
/// Provides clear conversion between different coordinate spaces
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub world: WorldPos,
    pub tile: TilePos,
}

/// World position in pixels
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldPos {
    pub x: f32,
    pub y: f32,
}

/// Tile position in grid coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
}

/// Screen position in pixels (for rendering)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPos {
    pub x: f32,
    pub y: f32,
}

/// Grid position for spatial partitioning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

/// Normalized device coordinates (-1 to 1)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NDC {
    pub x: f32,
    pub y: f32,
}

/// Different coordinate spaces in the game
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSpace {
    World,      // World coordinates (pixels)
    Tile,       // Tile coordinates (grid)
    Screen,     // Screen coordinates (pixels, for rendering)
    Grid,       // Grid coordinates (for spatial partitioning)
    MechFloor,  // Mech interior coordinates
    NDC,        // Normalized device coordinates
}

impl WorldPos {
    /// Create a new world position
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    /// Create world position from tile position
    pub fn from_tile(tile: TilePos) -> Self {
        Self {
            x: tile.x as f32 * TILE_SIZE,
            y: tile.y as f32 * TILE_SIZE,
        }
    }
    
    /// Create world position at the center of a tile
    pub fn from_tile_center(tile: TilePos) -> Self {
        Self {
            x: tile.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            y: tile.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        }
    }
    
    /// Convert to tile position (floor)
    pub fn to_tile(self) -> TilePos {
        TilePos {
            x: (self.x / TILE_SIZE).floor() as i32,
            y: (self.y / TILE_SIZE).floor() as i32,
        }
    }
    
    /// Convert to screen position (for rendering)
    pub fn to_screen(self, camera_offset: WorldPos) -> ScreenPos {
        ScreenPos {
            x: self.x - camera_offset.x,
            y: self.y - camera_offset.y,
        }
    }
    
    /// Convert to grid position for spatial partitioning
    pub fn to_grid(self, cell_size: f32) -> GridPos {
        GridPos {
            x: (self.x / cell_size).floor() as i32,
            y: (self.y / cell_size).floor() as i32,
        }
    }
    
    /// Convert to normalized device coordinates
    pub fn to_ndc(self, world_width: f32, world_height: f32) -> NDC {
        NDC {
            x: (self.x / world_width) * 2.0 - 1.0,
            y: (self.y / world_height) * 2.0 - 1.0,
        }
    }
    
    /// Calculate distance to another world position
    pub fn distance_to(self, other: WorldPos) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// Calculate squared distance (more efficient when comparing distances)
    pub fn distance_squared_to(self, other: WorldPos) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
    
    /// Check if this position is within world bounds
    pub fn is_in_world_bounds(self) -> bool {
        self.x >= 0.0 && self.x < ARENA_WIDTH_TILES as f32 * TILE_SIZE &&
        self.y >= 0.0 && self.y < ARENA_HEIGHT_TILES as f32 * TILE_SIZE
    }
    
    /// Check if this position is within mech floor bounds
    pub fn is_in_mech_floor_bounds(self) -> bool {
        self.x >= 0.0 && self.x < FLOOR_WIDTH_TILES as f32 * TILE_SIZE &&
        self.y >= 0.0 && self.y < FLOOR_HEIGHT_TILES as f32 * TILE_SIZE
    }
    
    /// Clamp position to world bounds
    pub fn clamp_to_world_bounds(self) -> WorldPos {
        WorldPos {
            x: self.x.max(0.0).min((ARENA_WIDTH_TILES as f32 * TILE_SIZE) - 1.0),
            y: self.y.max(0.0).min((ARENA_HEIGHT_TILES as f32 * TILE_SIZE) - 1.0),
        }
    }
    
    /// Clamp position to mech floor bounds
    pub fn clamp_to_mech_floor_bounds(self) -> WorldPos {
        WorldPos {
            x: self.x.max(0.0).min((FLOOR_WIDTH_TILES as f32 * TILE_SIZE) - 1.0),
            y: self.y.max(0.0).min((FLOOR_HEIGHT_TILES as f32 * TILE_SIZE) - 1.0),
        }
    }
    
    /// Linear interpolation between two world positions
    pub fn lerp(self, other: WorldPos, t: f32) -> WorldPos {
        WorldPos {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
    
    /// Get the normalized direction vector to another position
    pub fn direction_to(self, other: WorldPos) -> WorldPos {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let length = (dx * dx + dy * dy).sqrt();
        if length > 0.0 {
            WorldPos { x: dx / length, y: dy / length }
        } else {
            WorldPos { x: 0.0, y: 0.0 }
        }
    }
    
    /// Get the magnitude (length) of this position as a vector
    pub fn magnitude(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    /// Normalize this position as a vector
    pub fn normalize(self) -> WorldPos {
        let mag = self.magnitude();
        if mag > 0.0 {
            WorldPos { x: self.x / mag, y: self.y / mag }
        } else {
            WorldPos { x: 0.0, y: 0.0 }
        }
    }
}

impl TilePos {
    /// Create a new tile position
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    /// Create tile position from world position
    pub fn from_world(world: WorldPos) -> Self {
        Self {
            x: (world.x / TILE_SIZE).floor() as i32,
            y: (world.y / TILE_SIZE).floor() as i32,
        }
    }
    
    /// Convert to world position (top-left corner of tile)
    pub fn to_world(self) -> WorldPos {
        WorldPos {
            x: self.x as f32 * TILE_SIZE,
            y: self.y as f32 * TILE_SIZE,
        }
    }
    
    /// Convert to world position (center of tile)
    pub fn to_world_center(self) -> WorldPos {
        WorldPos {
            x: self.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            y: self.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
        }
    }
    
    /// Calculate Manhattan distance to another tile position
    pub fn manhattan_distance_to(self, other: TilePos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
    
    /// Calculate Euclidean distance to another tile position
    pub fn distance_to(self, other: TilePos) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// Check if this tile position is within world bounds
    pub fn is_in_world_bounds(self) -> bool {
        self.x >= 0 && self.x < ARENA_WIDTH_TILES &&
        self.y >= 0 && self.y < ARENA_HEIGHT_TILES
    }
    
    /// Check if this tile position is within mech floor bounds
    pub fn is_in_mech_floor_bounds(self) -> bool {
        self.x >= 0 && self.x < FLOOR_WIDTH_TILES &&
        self.y >= 0 && self.y < FLOOR_HEIGHT_TILES
    }
    
    /// Clamp tile position to world bounds
    pub fn clamp_to_world_bounds(self) -> TilePos {
        TilePos {
            x: self.x.max(0).min(ARENA_WIDTH_TILES - 1),
            y: self.y.max(0).min(ARENA_HEIGHT_TILES - 1),
        }
    }
    
    /// Clamp tile position to mech floor bounds
    pub fn clamp_to_mech_floor_bounds(self) -> TilePos {
        TilePos {
            x: self.x.max(0).min(FLOOR_WIDTH_TILES - 1),
            y: self.y.max(0).min(FLOOR_HEIGHT_TILES - 1),
        }
    }
    
    /// Get neighboring tile positions (4-directional)
    pub fn neighbors_4(self) -> [TilePos; 4] {
        [
            TilePos { x: self.x - 1, y: self.y },     // Left
            TilePos { x: self.x + 1, y: self.y },     // Right
            TilePos { x: self.x, y: self.y - 1 },     // Up
            TilePos { x: self.x, y: self.y + 1 },     // Down
        ]
    }
    
    /// Get neighboring tile positions (8-directional)
    pub fn neighbors_8(self) -> [TilePos; 8] {
        [
            TilePos { x: self.x - 1, y: self.y - 1 }, // Top-left
            TilePos { x: self.x, y: self.y - 1 },     // Top
            TilePos { x: self.x + 1, y: self.y - 1 }, // Top-right
            TilePos { x: self.x - 1, y: self.y },     // Left
            TilePos { x: self.x + 1, y: self.y },     // Right
            TilePos { x: self.x - 1, y: self.y + 1 }, // Bottom-left
            TilePos { x: self.x, y: self.y + 1 },     // Bottom
            TilePos { x: self.x + 1, y: self.y + 1 }, // Bottom-right
        ]
    }
    
    /// Create an offset tile position
    pub fn offset(self, dx: i32, dy: i32) -> TilePos {
        TilePos {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl ScreenPos {
    /// Create a new screen position
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    /// Convert to world position (add camera offset)
    pub fn to_world(self, camera_offset: WorldPos) -> WorldPos {
        WorldPos {
            x: self.x + camera_offset.x,
            y: self.y + camera_offset.y,
        }
    }
}

impl GridPos {
    /// Create a new grid position
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    /// Convert to world position (center of grid cell)
    pub fn to_world(self, cell_size: f32) -> WorldPos {
        WorldPos {
            x: self.x as f32 * cell_size + cell_size / 2.0,
            y: self.y as f32 * cell_size + cell_size / 2.0,
        }
    }
}

impl NDC {
    /// Create new normalized device coordinates
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    /// Convert to world position
    pub fn to_world(self, world_width: f32, world_height: f32) -> WorldPos {
        WorldPos {
            x: (self.x + 1.0) / 2.0 * world_width,
            y: (self.y + 1.0) / 2.0 * world_height,
        }
    }
}

/// Coordinate conversion utilities
pub mod conversion {
    use super::*;
    
    /// Convert between any two coordinate types
    pub fn convert(from: CoordinateSpace, to: CoordinateSpace, value: f32, value2: f32) -> (f32, f32) {
        match (from, to) {
            (CoordinateSpace::World, CoordinateSpace::Tile) => {
                let world_pos = WorldPos::new(value, value2);
                let tile_pos = world_pos.to_tile();
                (tile_pos.x as f32, tile_pos.y as f32)
            }
            (CoordinateSpace::Tile, CoordinateSpace::World) => {
                let tile_pos = TilePos::new(value as i32, value2 as i32);
                let world_pos = tile_pos.to_world();
                (world_pos.x, world_pos.y)
            }
            (CoordinateSpace::World, CoordinateSpace::Grid) => {
                let world_pos = WorldPos::new(value, value2);
                let grid_pos = world_pos.to_grid(TILE_SIZE);
                (grid_pos.x as f32, grid_pos.y as f32)
            }
            _ => (value, value2), // Same space or not implemented
        }
    }
    
    /// Check if a position is valid in a given coordinate space
    pub fn is_valid_in_space(space: CoordinateSpace, x: f32, y: f32) -> bool {
        match space {
            CoordinateSpace::World => {
                let pos = WorldPos::new(x, y);
                pos.is_in_world_bounds()
            }
            CoordinateSpace::Tile => {
                let pos = TilePos::new(x as i32, y as i32);
                pos.is_in_world_bounds()
            }
            CoordinateSpace::MechFloor => {
                let pos = WorldPos::new(x, y);
                pos.is_in_mech_floor_bounds()
            }
            CoordinateSpace::NDC => {
                x >= -1.0 && x <= 1.0 && y >= -1.0 && y <= 1.0
            }
            _ => true, // Screen and Grid don't have fixed bounds
        }
    }
}

// Arithmetic operations for WorldPos
impl Add for WorldPos {
    type Output = WorldPos;
    
    fn add(self, other: WorldPos) -> WorldPos {
        WorldPos {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for WorldPos {
    type Output = WorldPos;
    
    fn sub(self, other: WorldPos) -> WorldPos {
        WorldPos {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for WorldPos {
    type Output = WorldPos;
    
    fn mul(self, scalar: f32) -> WorldPos {
        WorldPos {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<f32> for WorldPos {
    type Output = WorldPos;
    
    fn div(self, scalar: f32) -> WorldPos {
        WorldPos {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

// Arithmetic operations for TilePos
impl Add for TilePos {
    type Output = TilePos;
    
    fn add(self, other: TilePos) -> TilePos {
        TilePos {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for TilePos {
    type Output = TilePos;
    
    fn sub(self, other: TilePos) -> TilePos {
        TilePos {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_tile_conversion() {
        let world_pos = WorldPos::new(64.0, 96.0);
        let tile_pos = world_pos.to_tile();
        
        assert_eq!(tile_pos.x, 2);
        assert_eq!(tile_pos.y, 3);
    }
    
    #[test]
    fn test_tile_to_world_conversion() {
        let tile_pos = TilePos::new(2, 3);
        let world_pos = tile_pos.to_world();
        
        assert_eq!(world_pos.x, 64.0);
        assert_eq!(world_pos.y, 96.0);
    }
    
    #[test]
    fn test_world_pos_distance() {
        let pos1 = WorldPos::new(0.0, 0.0);
        let pos2 = WorldPos::new(3.0, 4.0);
        
        assert_eq!(pos1.distance_to(pos2), 5.0);
    }
    
    #[test]
    fn test_tile_pos_manhattan_distance() {
        let pos1 = TilePos::new(0, 0);
        let pos2 = TilePos::new(3, 4);
        
        assert_eq!(pos1.manhattan_distance_to(pos2), 7);
    }
    
    #[test]
    fn test_bounds_checking() {
        let valid_world = WorldPos::new(100.0, 100.0);
        let invalid_world = WorldPos::new(-10.0, -10.0);
        
        assert!(valid_world.is_in_world_bounds());
        assert!(!invalid_world.is_in_world_bounds());
    }
    
    #[test]
    fn test_arithmetic_operations() {
        let pos1 = WorldPos::new(10.0, 20.0);
        let pos2 = WorldPos::new(5.0, 15.0);
        
        let sum = pos1 + pos2;
        assert_eq!(sum.x, 15.0);
        assert_eq!(sum.y, 35.0);
        
        let diff = pos1 - pos2;
        assert_eq!(diff.x, 5.0);
        assert_eq!(diff.y, 5.0);
        
        let scaled = pos1 * 2.0;
        assert_eq!(scaled.x, 20.0);
        assert_eq!(scaled.y, 40.0);
    }
}
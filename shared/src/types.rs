use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
}

impl TilePos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_world_pos(&self) -> WorldPos {
        WorldPos {
            x: self.x as f32 * crate::TILE_SIZE,
            y: self.y as f32 * crate::TILE_SIZE,
        }
    }

    pub fn distance_to(&self, other: &TilePos) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorldPos {
    pub x: f32,
    pub y: f32,
}

impl WorldPos {
    pub fn to_tile_pos(&self) -> TilePos {
        TilePos {
            x: (self.x / crate::TILE_SIZE).floor() as i32,
            y: (self.y / crate::TILE_SIZE).floor() as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_offset(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    ScrapMetal,
    ComputerComponents,
    Wiring,
    Batteries,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpgradeType {
    Laser,
    Projectile,
    Shield,
    Engine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StationType {
    WeaponLaser,
    WeaponProjectile,
    Engine,
    Shield,
    Repair,
    Electrical,
    Upgrade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerLocation {
    OutsideWorld(TilePos),
    InsideMech { mech_id: uuid::Uuid, floor: u8, pos: TilePos },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamId {
    Red,
    Blue,
}
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// UUID Type Aliases for Type Safety
// =============================================================================

pub type PlayerId = Uuid;
pub type MechId = Uuid;
pub type StationId = Uuid;
pub type ResourceId = Uuid;
pub type ProjectileId = Uuid;
pub type WeaponEffectId = Uuid;
pub type EntityId = Uuid;

// Re-export the unified coordinate types for backward compatibility
pub use crate::coordinates::{GridPos, MechInteriorPos, ScreenPos, TilePos, WorldPos, NDC};

// Add serde support to the coordinate types
impl Serialize for WorldPos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("WorldPos", 2)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for WorldPos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct WorldPosHelper {
            x: f32,
            y: f32,
        }

        let helper = WorldPosHelper::deserialize(deserializer)?;
        Ok(WorldPos::new(helper.x, helper.y))
    }
}

impl Serialize for TilePos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("TilePos", 2)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for TilePos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TilePosHelper {
            x: i32,
            y: i32,
        }

        let helper = TilePosHelper::deserialize(deserializer)?;
        Ok(TilePos::new(helper.x, helper.y))
    }
}

// Add backward compatibility methods
impl WorldPos {
    /// Legacy method for backward compatibility
    pub fn to_tile_pos(&self) -> TilePos {
        self.to_tile()
    }

    /// Legacy method for backward compatibility
    pub fn move_in_direction(&self, direction: Direction, speed: f32, delta_time: f32) -> Self {
        let (dx, dy) = direction.to_velocity();
        Self {
            x: self.x + dx * speed * delta_time,
            y: self.y + dy * speed * delta_time,
        }
    }
}

impl TilePos {
    /// Legacy method for backward compatibility
    pub fn to_world_pos(&self) -> WorldPos {
        self.to_world()
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

    pub fn to_velocity(&self) -> (f32, f32) {
        match self {
            Direction::Up => (0.0, -1.0),
            Direction::Down => (0.0, 1.0),
            Direction::Left => (-1.0, 0.0),
            Direction::Right => (1.0, 0.0),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StationType {
    WeaponLaser,
    WeaponProjectile,
    Engine,
    Shield,
    Repair,
    Electrical,
    Upgrade,
    Pilot,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlayerLocation {
    OutsideWorld(WorldPos),
    InsideMech {
        mech_id: MechId,
        pos: MechInteriorPos,
    },
}

impl PlayerLocation {
    /// Get the effective world position for this player location
    /// For InsideMech, requires mech world position for conversion
    pub fn world_pos(&self, mech_world_pos: Option<WorldPos>) -> WorldPos {
        match self {
            PlayerLocation::OutsideWorld(pos) => *pos,
            PlayerLocation::InsideMech { pos, .. } => {
                match mech_world_pos {
                    Some(mech_pos) => pos.to_world_with_mech(mech_pos),
                    None => pos.to_local_world(), // Fallback to local coordinates
                }
            }
        }
    }

    /// Get the effective tile position for this player location
    pub fn tile_pos(&self, mech_world_pos: Option<WorldPos>) -> TilePos {
        self.world_pos(mech_world_pos).to_tile()
    }

    /// Check if player is inside a specific mech
    pub fn is_inside_mech(&self, mech_id: MechId) -> bool {
        matches!(self, PlayerLocation::InsideMech { mech_id: id, .. } if *id == mech_id)
    }

    /// Get the mech ID if player is inside a mech
    pub fn mech_id(&self) -> Option<MechId> {
        match self {
            PlayerLocation::InsideMech { mech_id, .. } => Some(*mech_id),
            _ => None,
        }
    }

    /// Get the floor if player is inside a mech
    pub fn floor(&self) -> Option<u8> {
        match self {
            PlayerLocation::InsideMech { pos, .. } => Some(pos.floor()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamId {
    Red,
    Blue,
}

// Note: Old tile system (WorldTile, MechInteriorTile) has been replaced
// by the hybrid tile-entity system in tile_entity.rs
// Use TileMap, TileContent, StaticTile, and entity references instead

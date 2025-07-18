use serde::{Serialize, Deserialize};
use uuid::Uuid;

// Re-export the unified coordinate types for backward compatibility
pub use crate::coordinates::{WorldPos, TilePos, ScreenPos, GridPos, NDC};

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
    InsideMech { mech_id: Uuid, floor: u8, pos: WorldPos },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeamId {
    Red,
    Blue,
}

// Tile interaction system

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileProperties {
    pub walkable: bool,
    pub blocks_projectiles: bool,
    pub interaction: TileInteraction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TileInteraction {
    None,
    EnterMech { mech_id: Uuid },
    ExitMech { exit_position: TilePos },
    DropResource { mech_id: Uuid },
    OperateStation { station_id: Uuid },
    ChangeFloor { direction: i8 }, // -1 for down, +1 for up
}

// World tiles (outside of mechs)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorldTile {
    Grass,
    MechDoor { mech_id: Uuid },
    ResourceDropoff { mech_id: Uuid },
    Wall,
    Empty, // Out of bounds or uninitialized
}

impl WorldTile {
    pub fn properties(&self) -> TileProperties {
        match self {
            WorldTile::Grass => TileProperties {
                walkable: true,
                blocks_projectiles: false,
                interaction: TileInteraction::None,
            },
            WorldTile::MechDoor { mech_id, .. } => TileProperties {
                walkable: true,
                blocks_projectiles: false,
                interaction: TileInteraction::EnterMech { mech_id: *mech_id },
            },
            WorldTile::ResourceDropoff { mech_id } => TileProperties {
                walkable: true,
                blocks_projectiles: false,
                interaction: TileInteraction::DropResource { mech_id: *mech_id },
            },
            WorldTile::Wall => TileProperties {
                walkable: false,
                blocks_projectiles: true,
                interaction: TileInteraction::None,
            },
            WorldTile::Empty => TileProperties {
                walkable: false,
                blocks_projectiles: false,
                interaction: TileInteraction::None,
            },
        }
    }
}

// Interior mech tiles
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MechInteriorTile {
    Empty,
    Floor,
    Wall,
    Station(StationType),
    Ladder,
    ExitDoor { exit_position: TilePos }, // Door to exit the mech
}

impl MechInteriorTile {
    pub fn properties(&self, station_id: Option<Uuid>) -> TileProperties {
        match self {
            MechInteriorTile::Empty => TileProperties {
                walkable: false,
                blocks_projectiles: false,
                interaction: TileInteraction::None,
            },
            MechInteriorTile::Floor => TileProperties {
                walkable: true,
                blocks_projectiles: false,
                interaction: TileInteraction::None,
            },
            MechInteriorTile::Wall => TileProperties {
                walkable: false,
                blocks_projectiles: true,
                interaction: TileInteraction::None,
            },
            MechInteriorTile::Station(_) => TileProperties {
                walkable: true,
                blocks_projectiles: false,
                interaction: if let Some(id) = station_id {
                    TileInteraction::OperateStation { station_id: id }
                } else {
                    TileInteraction::None
                },
            },
            MechInteriorTile::Ladder => TileProperties {
                walkable: true,
                blocks_projectiles: false,
                interaction: TileInteraction::ChangeFloor { direction: 0 }, // Direction determined by context
            },
            MechInteriorTile::ExitDoor { exit_position } => TileProperties {
                walkable: true,
                blocks_projectiles: false,
                interaction: TileInteraction::ExitMech { exit_position: *exit_position },
            },
        }
    }
}
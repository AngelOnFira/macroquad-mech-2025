# Hybrid Tile-Entity System - Implementation Guide

## Overview

This document describes the implemented hybrid tile-entity system that combines simple tile enums for basic world elements (walls, floors) with an Entity Component System (ECS) for complex interactive objects (stations, turrets). This design provides performance for static geometry while maintaining flexibility for dynamic objects.

## Current Implementation Status

### ✅ Implemented
- Core tile system with `TileContent` enum
- Static tiles for walls, floors, windows, transitions
- Entity-based stations and turrets
- Raycasting vision system with window mechanics
- Layered floor system (Dwarf Fortress style)
- Continuous movement over tiles
- Client-server tile visual protocol
- Demo showcasing all features

### 🚧 In Progress
- Full server integration
- Unified world space coordinates
- Mech movement with interior

### 📋 Future
- Power/wiring systems
- Breakable tiles
- Atmospheric simulation
- Modding support

## Core Architecture

### Tile Map Structure

Located in `shared/src/tile_entity.rs`:

```rust
pub struct TileMap {
    tiles: HashMap<TilePos, TileContent>,
    mechs: HashMap<Uuid, MechTileMap>,
}

pub struct MechTileMap {
    floors: Vec<FloorMap>,
    position: TilePos,  // World position of mech
}

pub struct FloorMap {
    tiles: HashMap<TilePos, TileContent>,
}

pub enum TileContent {
    Empty,
    Static(StaticTile),
    Entity(Uuid),  // Reference to entity in storage
}
```

### Static Tiles

Simple tiles that don't need complex behavior:

```rust
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum StaticTile {
    // Floors
    MetalFloor,
    CargoFloor { wear: u8 },
    
    // Walls  
    MetalWall,
    ReinforcedWall,
    
    // Windows
    Window { facing: Direction },
    ReinforcedWindow { facing: Direction, tint: WindowTint },
    
    // Transitions
    TransitionZone { zone_id: u8, transition_type: TransitionType },
    
    // Basic infrastructure
    PowerConduit,
    DataCable,
}
```

### Entity Components

Located in `shared/src/components.rs`:

```rust
#[derive(Component)]
pub struct Position {
    pub tile: TilePos,
    pub world: WorldPos,
    pub floor: Option<u8>,
    pub mech_id: Option<Uuid>,
}

#[derive(Component)]
pub struct Station {
    pub station_type: StationType,
    pub interaction_range: f32,
    pub power_required: f32,
    pub operating: bool,
}

#[derive(Component)]
pub struct Turret {
    pub damage: f32,
    pub fire_rate: f32,
    pub range: f32,
    pub facing: Direction,
}
```

## Vision System

The vision system (`shared/src/vision.rs`) implements proper raycasting with line-of-sight:

```rust
pub struct VisionSystem {
    cache: HashMap<(Uuid, u64), VisibilityData>,
}

impl VisionSystem {
    pub fn calculate_visibility<S: ComponentStorage>(
        &mut self,
        viewer_id: Uuid,
        viewer_pos: WorldPos,
        max_range: f32,
        tile_map: &TileMap,
        storage: &S,
    ) -> &VisibilityData {
        // Performs raycasting to determine visible tiles
        // Windows provide extended vision cones
        // Mech interiors are dark by default
    }
}
```

### Key Features:
- **Raycasting**: True line-of-sight calculation
- **Window Mechanics**: Windows extend vision in directional cones
- **Dark Interiors**: Mech interiors require line-of-sight to see
- **Performance**: Caches visibility calculations per tick

## Layered Floor System

Inspired by Dwarf Fortress, floors are rendered as layers:

- **Ground Level (Z=0)**: The outside world
- **Mech Floors (Z=1,2,3)**: Interior floors of mechs
- **Same XY Coordinates**: All floors exist at same world position
- **Layer Transitions**: Smooth fades between floors

Example from the demo:
```rust
enum LayerType {
    Ground,         // The world outside
    MechFloor(u8),  // Floor 0, 1, 2 of a mech
}
```

## Movement System

Movement is continuous over tiles, not discrete:

```rust
// Player position is in world coordinates (pixels)
player_x: f32,
player_y: f32,

// Movement is smooth
player_x += velocity_x * delta_time;
player_y += velocity_y * delta_time;

// Collision checks against tile grid
let tile_x = (player_x / TILE_SIZE).floor() as i32;
let tile_y = (player_y / TILE_SIZE).floor() as i32;
```

## Client-Server Protocol

The server sends simplified tile visuals to clients:

```rust
#[derive(Serialize, Deserialize)]
pub enum TileVisual {
    // Static visuals
    Floor { material: Material, wear: u8 },
    Wall { material: Material },
    Window { broken: bool, facing: Direction },
    
    // Entity visuals
    Station { station_type: StationType, active: bool },
    Turret { facing: Direction, firing: bool },
    
    // Effects
    TransitionFade { progress: f32 },
}
```

## Entity Storage System

Located in `server/src/entity_storage.rs`:

```rust
pub struct EntityStorage {
    // Component storage
    positions: HashMap<Uuid, Position>,
    stations: HashMap<Uuid, Station>,
    turrets: HashMap<Uuid, Turret>,
    
    // Spatial indexing for performance
    entities_by_position: HashMap<TilePos, Vec<Uuid>>,
    entities_by_mech: HashMap<Uuid, Vec<Uuid>>,
}
```

## Integration Example

Creating a mech with stations:

```rust
// Create mech tile map
let mut mech_map = MechTileMap::new(position);

// Add static tiles (walls, floors)
for floor in 0..3 {
    mech_map.floors[floor].set_static_tile(
        TilePos::new(0, 0), 
        StaticTile::MetalWall
    );
}

// Create station entity
let station_id = Uuid::new_v4();
entities.spawn_station(
    station_id,
    Position { tile: TilePos::new(5, 5), floor: Some(0), mech_id: Some(mech_id) },
    Station { station_type: StationType::Engine, operating: false, ... }
);

// Link entity to tile
mech_map.floors[0].set_entity_tile(TilePos::new(5, 5), station_id);
```

## Performance Benefits

1. **Static Tiles Are Fast**: Simple enum comparisons for walls/floors
2. **Spatial Indexing**: O(1) entity lookups by position
3. **Vision Caching**: Visibility only recalculated when needed
4. **Minimal Network Traffic**: Only changed tiles sent to clients

## Demo

Run the interactive demo to see the system in action:

```bash
just dev-demo
# Open http://localhost:8080/demo.html
```

Features demonstrated:
- Layered floors with transitions
- Continuous movement
- Raycasting vision with darkness
- Window vision mechanics
- Station and turret entities

## Future Enhancements

### Power System
```rust
#[derive(Component)]
pub struct PowerNode {
    pub connections: Vec<Uuid>,
    pub flow: f32,
}
```

### Breakable Tiles
```rust
#[derive(Component)]
pub struct Breakable {
    pub health: f32,
    pub break_effects: Vec<BreakEffect>,
}
```

### Modding Support
- Entity templates loaded from JSON
- Scriptable component for custom behavior
- Dynamic component registration

## Migration Notes

When migrating old code:
1. Replace `WorldTile`/`MechInteriorTile` with `TileContent`
2. Move complex tiles (stations) to entities
3. Use `TileVisual` for client rendering
4. Update movement to use continuous positions
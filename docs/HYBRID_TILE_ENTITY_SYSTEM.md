# Hybrid Tile-Entity System Design

## Overview

This document describes a hybrid approach that combines simple tile enums for basic world elements with an Entity Component System (ECS) for complex interactive objects. This design provides a fast path to a working game while maintaining extensibility for advanced features like modding, wiring systems, and dynamic object placement.

## Core Architecture

### Tile Map Structure

```rust
pub struct TileMap {
    // Simple tiles indexed by position
    static_tiles: HashMap<TilePos, StaticTile>,
    
    // Entity references for complex tiles
    entity_tiles: HashMap<TilePos, EntityId>,
    
    // Spatial index for fast lookups
    spatial_index: SpatialIndex,
    
    // Mech-relative tiles
    mech_tiles: HashMap<Uuid, MechTileMap>,
}

pub struct MechTileMap {
    floors: Vec<FloorMap>,
    mech_entity: EntityId,  // The mech itself is an entity
}

pub struct FloorMap {
    static_tiles: HashMap<TilePos, StaticTile>,
    entity_tiles: HashMap<TilePos, EntityId>,
}
```

### Content Types

```rust
pub enum TileContent {
    Empty,
    Static(StaticTile),
    Entity(EntityId),
}

// Simple tiles that don't need complex behavior
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

#[derive(Clone, Copy, Debug)]
pub enum TransitionType {
    MechEntrance { stage: u8 },  // 0 = first tile, 1 = second tile
    StairUp { stage: u8 },
    StairDown { stage: u8 },
    Ladder,
}
```

### Entity Components for Complex Objects

```rust
// Core components that tiles might have
#[derive(Component)]
pub struct Position {
    pub tile: TilePos,
    pub world: WorldPos,
    pub floor: Option<u8>,  // None = outside, Some(n) = mech floor
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
    pub ammo: u32,
    pub target_mode: TargetMode,
    pub current_target: Option<EntityId>,
}

#[derive(Component)]
pub struct PowerNode {
    pub max_throughput: f32,
    pub current_load: f32,
    pub connections: Vec<EntityId>,
    pub network_id: Uuid,
}

#[derive(Component)]
pub struct Breakable {
    pub health: f32,
    pub max_health: f32,
    pub armor: f32,
    pub break_effects: Vec<BreakEffect>,
}

#[derive(Component)]
pub struct Renderable {
    pub sprite: SpriteId,
    pub layer: RenderLayer,
    pub color_modulation: Color,
    pub animation_state: Option<AnimationState>,
}

// For moddable content
#[derive(Component)]
pub struct Scriptable {
    pub script_id: String,
    pub state: HashMap<String, Value>,
}
```

## Static Tile Behaviors

Simple tiles have fixed behaviors defined in code:

```rust
impl StaticTile {
    pub fn is_walkable(&self) -> bool {
        match self {
            StaticTile::MetalFloor | StaticTile::CargoFloor { .. } => true,
            StaticTile::TransitionZone { .. } => true,
            StaticTile::PowerConduit | StaticTile::DataCable => true,
            _ => false,
        }
    }
    
    pub fn blocks_vision(&self) -> bool {
        match self {
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
    
    pub fn on_enter(&self, actor: EntityId) -> Option<TileEvent> {
        match self {
            StaticTile::TransitionZone { zone_id, transition_type } => {
                Some(TileEvent::BeginTransition {
                    actor,
                    zone_id: *zone_id,
                    transition_type: *transition_type,
                })
            }
            _ => None,
        }
    }
}
```

## System Integration

### Movement System

```rust
pub fn handle_movement(
    world: &World,
    tile_map: &TileMap,
    entity: EntityId,
    new_pos: WorldPos,
) -> Result<(), MovementError> {
    // Check static tile at destination
    let tile_pos = new_pos.to_tile_pos();
    if let Some(static_tile) = tile_map.get_static_at(tile_pos) {
        if !static_tile.is_walkable() {
            return Err(MovementError::Blocked);
        }
    }
    
    // Check entities at destination
    if let Some(entity_id) = tile_map.get_entity_at(tile_pos) {
        // Query entity components
        if world.get::<Solid>(entity_id).is_some() {
            return Err(MovementError::Blocked);
        }
    }
    
    Ok(())
}
```

### Vision System

```rust
pub fn calculate_vision(
    world: &World,
    tile_map: &TileMap,
    viewer_pos: WorldPos,
    max_range: f32,
) -> VisibilityMap {
    let mut visible = HashSet::new();
    
    // Raycast implementation
    for angle in 0..360 {
        let mut ray = Ray::new(viewer_pos, angle as f32);
        let mut attenuation = 0.0;
        
        while ray.length < max_range && attenuation < 1.0 {
            let check_pos = ray.current_pos();
            let tile_pos = check_pos.to_tile_pos();
            
            // Check static tiles
            if let Some(static_tile) = tile_map.get_static_at(tile_pos) {
                attenuation += static_tile.vision_attenuation();
                if static_tile.blocks_vision() {
                    break;
                }
            }
            
            // Check entity tiles
            if let Some(entity_id) = tile_map.get_entity_at(tile_pos) {
                if let Some(opaque) = world.get::<Opaque>(entity_id) {
                    attenuation += opaque.attenuation;
                    if opaque.blocks_completely {
                        break;
                    }
                }
            }
            
            if attenuation < 1.0 {
                visible.insert(tile_pos);
            }
            
            ray.advance(0.5);
        }
    }
    
    VisibilityMap { visible_tiles: visible }
}
```

## Client-Server Communication

### Server Authority

```rust
// Server has full tile and entity data
pub struct ServerWorld {
    tile_map: TileMap,
    ecs: World,  // Full ECS with all components
}

// Server sends updates
pub enum TileUpdate {
    StaticTileChanged {
        pos: TilePos,
        new_tile: Option<StaticTile>,
    },
    EntityTileChanged {
        pos: TilePos,
        entity_id: Option<EntityId>,
        visual: Option<EntityVisual>,
    },
}
```

### Client Representation

```rust
// Client receives simplified view
#[derive(Serialize, Deserialize)]
pub struct ClientTile {
    pub visual: TileVisual,
    pub walkable: bool,  // For prediction
}

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

## Modding Support

### Entity Definitions

```json
{
  "entity_templates": {
    "plasma_turret": {
      "components": {
        "Turret": {
          "damage": 50,
          "fire_rate": 0.5,
          "range": 15,
          "ammo": 100
        },
        "PowerConsumer": {
          "idle_draw": 10,
          "active_draw": 100
        },
        "Scriptable": {
          "script_id": "plasma_turret_ai"
        }
      }
    }
  }
}
```

### Script Interface

```rust
pub trait ModScript {
    fn on_spawn(&mut self, entity: EntityId, world: &World);
    fn on_update(&mut self, entity: EntityId, world: &World, dt: f32);
    fn on_interact(&mut self, entity: EntityId, actor: EntityId, world: &World);
}
```

## Migration Path

### Phase 1: Current Prototype
- Use `StaticTile` for walls, floors, windows
- Stations are entities with `Station` component
- Basic movement and vision with static tiles

### Phase 2: Enhanced Stations
- Add `PowerNode` components to stations
- Implement basic wiring with entity connections
- Add turrets as entities

### Phase 3: Full Modding
- Implement `Scriptable` component
- Load entity definitions from JSON
- Add component reflection for dynamic properties

### Phase 4: Advanced Features
- Complex damage model with `Breakable`
- Atmospheric simulation
- Full electrical/fluid networks

## Example: Adding a New Station Type

With this hybrid system, adding a new station is straightforward:

```rust
// 1. Add to StationType enum (if hardcoded)
pub enum StationType {
    // ... existing types ...
    OxygenGenerator,  // New!
}

// 2. Create entity with components
let oxygen_gen = world.spawn((
    Position { tile: pos, floor: Some(1), mech_id: Some(mech_id) },
    Station { 
        station_type: StationType::OxygenGenerator,
        power_required: 50.0,
        operating: false,
    },
    PowerConsumer { draw: 50.0 },
    OxygenProducer { rate: 10.0 },  // Custom component
    Renderable { sprite: SpriteId::OxygenGen, layer: RenderLayer::Stations },
));

// 3. No other code changes needed!
```

## Benefits of This Approach

1. **Simple Things Stay Simple**: Walls and floors don't need entity overhead
2. **Complex Things Are Flexible**: Stations, turrets use full ECS
3. **Gradual Complexity**: Can start simple and add components as needed
4. **Modder Friendly**: New entity types don't require core changes
5. **Performance**: Static tiles are fast to query and render
6. **Network Efficient**: Only send what changed, not full entity state
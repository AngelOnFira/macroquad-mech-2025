# Mech Interior System Design Document

## Current Implementation Status

✅ **Implemented**: Layered floor system, raycasting vision, continuous movement, window mechanics, dark interiors
🚧 **In Progress**: Server integration, mech movement with interior
📋 **Future**: Power systems, damage states, atmospheric simulation

## Vision Statement

Mechs are moving fortresses that players can enter and operate from within. The interior exists in world space, moving with the mech, creating a seamless experience where players feel like they're inside a vehicle traveling through the world rather than being teleported to a separate dimension.

## Core Concepts

### 1. Unified World Space
- Mech interiors exist at all times in world coordinates
- Interior tiles move with the mech as it travels
- No separate "interior dimension" - everything is part of the same world
- Player positions become relative to the mech when inside

### 2. Vision and Fog of War
- Interior spaces are dark/black by default when inside
- Windows provide limited vision to the outside world
- Vision cones from windows based on player proximity and angle
- Outside world visible through windows shows relative movement
- Strategic gameplay: players can hide inside mechs

### 3. Seamless Transitions
- Smooth fading between floors when using stairs/ladders
- Continuous movement system (not tile-to-tile jumping)
- Layer-based rendering with Z-level management
- No jarring teleportation feeling

## Mech Layout (As Implemented)

### Layered Floor System
- All floors exist at same X,Y coordinates (Dwarf Fortress style)
- Z-levels: Ground (0), Mech Floor 0 (1), Mech Floor 1 (2), Mech Floor 2 (3)
- Smooth transitions between floors with alpha fading

### Ground Floor (Floor 0)
- 30x20 tile space (FLOOR_WIDTH_TILES x FLOOR_HEIGHT_TILES)
- Entrance door at center bottom
- Ladders connecting to upper floors
- Station positions for basic operations

### Middle Floor (Floor 1)
- Engineering and support stations
- Windows providing directional outside visibility
- Connected via ladders to floors above and below

### Upper Floor (Floor 2)
- Command deck with pilot station
- Turret control stations
- Strategic window placement for visibility

### Future Considerations
- Loadable mech layouts from definition files
- Customizable interior configurations
- Electrical/system routing (inspired by Barotrauma)
- Damage states affecting interior tiles

### Tile Types (Hybrid System)

#### Static Tiles (Simple Enums)
1. **Structural**
   - `MetalWall` - Blocks movement and vision
   - `MetalFloor` - Basic walkable surface
   - `Window { facing: Direction }` - Allows vision in specific direction
   - `Door` - Can be opened/closed

2. **Transitions**
   - `Ladder` - Connects floors vertically
   - `TransitionZone` - Entry/exit points

3. **Infrastructure**
   - `PowerConduit` - Future power routing
   - `DataCable` - Future data connections

#### Entity-Based Tiles (Complex Objects)
1. **Stations**
   - Engine Control
   - Weapon Systems
   - Shield Generator
   - Navigation
   - Turret Control

2. **Turrets**
   - Directional weapons
   - Damage and fire rate stats
   - Range limitations

## Visibility System (Implemented)

### Raycasting Engine
```rust
// From shared/src/vision.rs
pub fn cast_ray(start: WorldPos, end: WorldPos, tile_map: &TileMap) -> Vec<TilePos>
```
- True line-of-sight calculation using DDA algorithm
- Mech interiors are dark by default (no ambient light)
- Only tiles with direct line-of-sight are visible
- Performance optimized with visibility caching

### Window Vision Mechanics (Implemented)
- Windows only provide vision when you can see the window itself
- Directional cone extends vision outside based on window facing
- Cone angle: 60 degrees centered on window direction
- Extended range: 150 units beyond normal vision
- Multiple windows create overlapping vision areas

### Vision Constants
- `BASE_VISION_RANGE`: 100.0 units
- `WINDOW_VISION_EXTENSION`: 150.0 units
- `WINDOW_CONE_ANGLE`: 60 degrees

### Rendering System (As Implemented)

#### Layer Management
```rust
enum LayerType {
    Ground,         // Z=0: The world outside
    MechFloor(u8),  // Z=1,2,3: Interior floors
}
```

#### Rendering Order
1. **Tile Rendering**: Based on current layer with alpha transitions
2. **Vision Overlay**: Dark tiles outside line-of-sight
3. **Entity Rendering**: Players, stations, turrets
4. **UI Elements**: Debug info, controls

#### Transition Effects
- Smooth alpha fade between floors (300ms)
- Current floor at 100% opacity
- Other floors fade out based on distance

## Implementation Status

### ✅ Phase 1: Hybrid Tile System (COMPLETE)
- Implemented `TileContent` enum with Static/Entity variants
- Basic tile properties working
- Coordinate systems established

### ✅ Phase 2: Interior Navigation (COMPLETE)
- Layered floor system implemented
- Ladder-based floor transitions
- Continuous movement over tiles

### ✅ Phase 3: Visibility System (COMPLETE)
- Full raycasting implementation
- Window vision cones working
- Dark interiors with proper line-of-sight

### 🚧 Phase 4: Server Integration (IN PROGRESS)
- Need to integrate with game server
- Synchronize mech positions
- Handle multiplayer visibility

### 📋 Phase 5: Advanced Features (FUTURE)
- Power and wiring systems
- Damage states
- Atmospheric simulation
- Loadable mech layouts

## Technical Implementation Details

### Performance Optimizations
- HashMap-based tile storage for O(1) lookups
- Visibility caching per frame/tick
- Spatial indexing for entity queries
- Only visible tiles sent to renderer

### Key Code Locations
- **Tile System**: `shared/src/tile_entity.rs`
- **Vision System**: `shared/src/vision.rs`
- **Components**: `shared/src/components.rs`
- **Demo**: `client/src/bin/demo.rs`
- **Entity Storage**: `server/src/entity_storage.rs`

### Coordinate Systems
```rust
// Tile coordinates (grid-based)
pub struct TilePos { pub x: i32, pub y: i32 }

// World coordinates (pixel-based)
pub struct WorldPos { pub x: f32, pub y: f32 }

// Conversion
const TILE_SIZE: f32 = 16.0;
```

## Running the Demo

```bash
# Build and run the hybrid tile system demo
just dev-demo
# Then open http://localhost:8080/demo.html
```

### Demo Controls
- **WASD**: Move player
- **1/2/3**: Switch between floors
- **V**: Toggle vision system
- **Tab**: Cycle current mech layer

## Design Inspirations
- **Dwarf Fortress**: Z-level layering system
- **Barotrauma**: Submarine interiors with complex systems
- **FTL**: Ship management and combat
- **Space Station 13**: Detailed interior simulation
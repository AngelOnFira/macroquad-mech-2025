# Mech Interior System Design Document

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
- 2-tile transition zones for entrances and stairs
- First tile initiates fade, second tile completes transition
- Smooth visual blend between exterior/interior views
- No jarring teleportation feeling

## Mech Layout

### Ground Floor (Cargo Bay)
- 10x10 tile space matching exterior footprint
- Entry area with 2-tile wide door at bottom center
- Walled-off resource acceptance area
- Stairs (2-tile transition) leading to upper floor
- Mostly open space for future cargo/equipment

### Upper Floor (Command Deck)
- Station positions around the perimeter
- Windows providing outside visibility
- Central area for movement
- Pilot station with enhanced viewing capabilities

### Future Considerations
- Loadable mech layouts from definition files
- Customizable interior configurations
- Electrical/system routing (inspired by Barotrauma)
- Damage states affecting interior tiles

## Tile System Architecture

### Unified Tile Abstraction

```rust
pub trait Tile {
    // Rendering
    fn render(&self, context: &RenderContext) -> RenderCommand;
    fn get_visibility(&self) -> Visibility;
    fn affects_light(&self) -> bool;
    
    // Interaction
    fn can_walk_through(&self) -> bool;
    fn on_enter(&mut self, entity: EntityId) -> Vec<GameEvent>;
    fn on_exit(&mut self, entity: EntityId) -> Vec<GameEvent>;
    fn on_interact(&mut self, entity: EntityId) -> Vec<GameEvent>;
    
    // Physics
    fn blocks_projectiles(&self) -> bool;
    fn has_collision(&self) -> bool;
    
    // Relationships
    fn affects_neighbors(&self) -> bool;
    fn on_neighbor_change(&mut self, direction: Direction, neighbor: &TileType);
}
```

### Tile Types

1. **Structural Tiles**
   - Wall (opaque, blocks movement/projectiles)
   - Window (transparent, blocks movement, allows vision)
   - Floor (walkable base tile)
   - Door (state-based: open/closed)

2. **Transition Tiles**
   - Entrance (2-tile sequence, triggers interior/exterior transition)
   - Stairs (2-tile sequence, triggers floor transition)
   - Ladder (single tile, manual floor change)

3. **Functional Tiles**
   - Station (interactive, tied to ship systems)
   - Resource Dropoff (accepts carried resources)
   - Power Conduit (future: electrical routing)
   - Vent (future: atmosphere management)

4. **World Tiles**
   - Grass (basic terrain)
   - Rock (obstacles)
   - Empty (void/out of bounds)

### Coordinate Systems

```rust
pub struct WorldPos {
    x: f32,
    y: f32,
}

pub struct MechRelativePos {
    mech_id: Uuid,
    local_x: f32,  // Relative to mech origin
    local_y: f32,
    floor: u8,
}

impl MechRelativePos {
    fn to_world_pos(&self, mech_position: WorldPos, mech_rotation: f32) -> WorldPos {
        // Transform relative coordinates to world space
        // Account for mech position and rotation
    }
}
```

## Visibility System

### Raycasting for Line of Sight
- Cast rays from player position to determine visible tiles
- Windows allow rays to pass through with attenuation
- Walls block rays completely
- Dynamic visibility updates as player moves

### Window Vision Mechanics
- Base visibility radius when inside (e.g., 5 tiles)
- Extended visibility through windows (e.g., +10 tiles in direction)
- Visibility cone based on window facing and player angle
- Multiple windows can overlap visibility areas

### Rendering Layers
1. **Base World Layer**: Terrain and exterior tiles
2. **Mech Exterior Layer**: Mech hulls and external features
3. **Mech Interior Layer**: Interior tiles (only visible when conditions met)
4. **Entity Layer**: Players, resources, effects
5. **UI Layer**: Station interfaces, pilot controls

## Implementation Phases

### Phase 1: Unified Tile System
- Merge WorldTile and MechInteriorTile into single trait-based system
- Implement basic tile properties and interactions
- Set up coordinate transformation system

### Phase 2: Basic Interior Navigation
- Implement mech-relative positioning
- Add entrance/exit transitions
- Basic floor switching with stairs

### Phase 3: Visibility System
- Implement raycasting for interior visibility
- Add window tiles with vision cones
- Black fog for non-visible areas

### Phase 4: Enhanced Interactions
- Smooth transition effects
- Advanced station interactions
- Resource management improvements

### Phase 5: Advanced Features
- Loadable mech layouts
- Damage and atmosphere systems
- Electrical routing

## Technical Considerations

### Performance
- Efficient tile lookup with spatial indexing
- Visibility calculation caching
- Only render visible tiles
- LOD system for distant mechs

### Networking
- Sync mech positions and interior states
- Efficient visibility updates
- Handle player state transitions

### Future Extensibility
- Plugin system for custom tile types
- Moddable mech layouts
- Scriptable tile behaviors
- Integration with damage/atmosphere systems

## Inspiration Sources
- **Barotrauma**: Submarine interiors with complex systems
- **FTL**: Ship management and combat
- **Space Station 13**: Detailed interior simulation
- **Lovers in a Dangerous Spacetime**: Cooperative ship operation
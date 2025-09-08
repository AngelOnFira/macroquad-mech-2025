name: "Dynamic Mech Floor Rendering System"
description: |

## Purpose of this Template

This PRP specifies the implementation of a dynamic mech floor rendering system that allows the server to describe complete mech interiors to clients for accurate rendering, replacing the current minimal floor information approach.

---

## Goal

Replace the current mech rendering system (server: 6x20 with minimal client info → client: 10x10 basic floors) with a dynamic system where the server sends complete floor tile data to clients, enabling accurate multi-floor mech interiors with proper station placement and floor transitions.

## Why

- **Enhanced Player Experience**: Players will see detailed, consistent mech interiors instead of simplified representations
- **Flexible Mech Design**: Enables future customization of mech layouts and room configurations  
- **Accurate Spatial Awareness**: Players can better understand mech layout and navigate between floors effectively
- **Foundation for Expansion**: Sets up architecture for future features like mech customization, damage visualization, and varied mech sizes

## What

A complete overhaul of mech representation and rendering that includes:

- **Server-side mech floor storage**: Each mech stores 3 floors using HashMap-based tile storage for maximum flexibility and scalability
- **Dynamic floor data transmission**: Server sends floor layouts to clients when they enter mechs
- **Multi-tile station support**: Stations can occupy multiple tiles (e.g., 2x2 control consoles)
- **Floor transition system**: Special stairway tiles that instantly transport players between floors
- **Per-floor rendering**: Clients render only the player's current floor with other players visible only on same floor
- **Procedural floor generation**: Initial implementation generates example floor layouts with stations placed dynamically

## Endpoints/APIs to Implement

**MechFloorData** – Server → Client – Send complete mech layouts to all players at game start
- All 3 floors of mech interior data (HashMap-based tile storage)
- Station entity references and multi-tile mappings  
- Stairway positions and target floors
- **Rationale**: Players outside mechs can see the bottom floor through windows/doors, so all players need access to floor 0 data for proper world rendering

**FloorTransition** – Client → Server – Request floor change via stairway
- Current position
- Target floor ID
- Validation of stairway access

**MechInteriorUpdate** – Server → Client – Real-time updates to floor tiles *(Future scope - not implemented in initial version)*
- Modified tile positions
- New tile data  
- Station changes/damage
- **Note**: Initial implementation will focus on static floor layouts; dynamic updates will be added later

## Current Directory Structure

```
mech-battle-arena/
├── shared/
│   ├── src/
│   │   ├── types.rs (Mech, Player structs)
│   │   ├── tile_entity.rs (Tile enum)
│   │   ├── messages.rs (network protocol)
│   │   └── coordinates.rs (position types)
├── server/
│   ├── src/
│   │   ├── game.rs (core game logic)
│   │   └── entity_storage.rs (entity management)
└── client/
    └── src/
        ├── main.rs (client entry point)
        └── rendering/ (rendering systems)
```

## Proposed Directory Structure

```
mech-battle-arena/
├── shared/
│   ├── src/
│   │   ├── types.rs (enhanced Mech with floor data)
│   │   ├── tile_entity.rs (Stairway tile, multi-tile stations)
│   │   ├── messages.rs (MechFloorData, FloorTransition messages)
│   │   ├── coordinates.rs (FloorPos, MechInteriorPos types)
│   │   └── mech_floor.rs (NEW - floor layout structures)
├── server/
│   ├── src/
│   │   ├── game.rs (floor transition logic)
│   │   ├── entity_storage.rs (multi-tile entity storage)
│   │   └── mech_generation.rs (NEW - procedural floor generation)
└── client/
    └── src/
        ├── main.rs (floor rendering state)
        ├── rendering/
        │   ├── mech_interior.rs (NEW - floor rendering)
        │   └── multi_tile_stations.rs (NEW - station rendering)
        └── floor_manager.rs (NEW - floor data management)
```

## Files to Reference

- `shared/src/types.rs` (read_only) Current Mech struct and player positioning
- `shared/src/tile_entity.rs` (read_only) Existing Tile enum patterns for extension
- `shared/src/messages.rs` (read_only) Network message patterns
- `server/src/game.rs` (read_only) Current mech interaction and player movement logic
- `client/src/main.rs` (read_only) Current rendering loop and mech display logic
- `shared/src/coordinates.rs` (read_only) Position type patterns for MechInteriorPos

## Files to Implement (concept)

### Core Data Structures

1. `shared/src/mech_floor.rs` - Floor layout data structures

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MechFloor {
    pub tiles: HashMap<TilePos, Tile>, // Flexible HashMap storage for any size/shape
    pub stations: HashMap<TilePos, EntityId>, // Multi-tile station mappings
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MechInterior {
    pub floors: [MechFloor; 3],
    pub current_occupants: HashMap<PlayerId, u8>, // player -> floor_id
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum StairwayDirection {
    Up,
    Down,
}
```

2. `shared/src/coordinates.rs` - Enhanced position types

```rust
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct MechInteriorPos {
    pub floor: u8,
    pub tile_pos: TilePos,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]  
pub struct FloorPos {
    pub x: u8,
    pub y: u8,
}
```

### Server Implementation

1. `server/src/mech_generation.rs` - Procedural floor generation

```rust
pub fn generate_basic_floors() -> MechInterior {
    let mut interior = MechInterior::default();
    
    // Generate floor 0 (engine room)
    place_station(&mut interior.floors[0], StationType::Engine, TilePos::new(4, 4), Size::new(2, 2));
    place_stairway(&mut interior.floors[0], TilePos::new(8, 8), StairwayDirection::Up);
    
    // Generate floor 1 (bridge) 
    place_station(&mut interior.floors[1], StationType::Helm, TilePos::new(4, 2), Size::new(2, 1));
    place_stairway(&mut interior.floors[1], TilePos::new(8, 8), StairwayDirection::Down);
    place_stairway(&mut interior.floors[1], TilePos::new(1, 1), StairwayDirection::Up);
    
    interior
}
```

2. `server/src/game.rs` - Floor transition logic

```rust
fn handle_floor_transition(&mut self, player_id: PlayerId, target_floor: u8) {
    if let Some(player) = self.players.get_mut(&player_id) {
        if let Some(mech_id) = player.current_mech {
            // Update player's floor in mech
            if let Some(mech) = self.mechs.get_mut(&mech_id) {
                mech.interior.current_occupants.insert(player_id, target_floor);
                player.position.floor = target_floor;
                
                // Send updated floor data
                self.send_mech_floor_data(player_id, mech_id, target_floor);
            }
        }
    }
}
```

### Client Implementation

1. `client/src/floor_manager.rs` - Floor data management

```rust
pub struct FloorManager {
    pub current_floors: HashMap<MechId, MechInterior>,
    pub active_floor: Option<u8>,
}

impl FloorManager {
    pub fn update_floor_data(&mut self, mech_id: MechId, floor_data: MechInterior) {
        self.current_floors.insert(mech_id, floor_data);
    }
    
    pub fn get_current_floor_tiles(&self, mech_id: MechId) -> Option<&HashMap<TilePos, Tile>> {
        let floor_id = self.active_floor?;
        self.current_floors.get(&mech_id)?.floors.get(floor_id as usize).map(|f| &f.tiles)
    }
}
```

2. `client/src/rendering/mech_interior.rs` - Floor rendering

```rust
pub fn render_mech_floor(floor_tiles: &HashMap<TilePos, Tile>, camera_offset: Vec2) {
    for (tile_pos, tile) in floor_tiles {
        let world_pos = tile_pos.to_world();
        render_tile(*tile, world_pos + camera_offset);
    }
}
```

## World Rendering Integration

### Complete Rendering Pipeline
The dynamic mech floor system integrates with the existing world rendering to create seamless indoor-outdoor visualization:

1. **Layered Rendering Order**:
   - Render grass/terrain tiles first (world background)
   - Render mech exteriors and floor 0 interiors (all mechs visible to all players)
   - Render players and entities on current floor
   - Apply fog of war and vision occlusion last

2. **Multi-Floor Visibility**:
   - **Outside mechs**: Players can see floor 0 interiors through doors/windows when not occluded
   - **Inside mechs**: Players only see their current floor + other players on same floor
   - **Transition areas**: Door tiles provide visual connection between interior and exterior

3. **Vision Integration**:
   - Existing raycasting system works with mech interior tiles
   - Windows in mech walls allow visibility into floor 0 from outside
   - Door tiles create natural entry/exit visual flow
   - Vision boundaries respect floor transitions (stairways block cross-floor vision)

4. **Seamless World Experience**:
   - Player approaching mech sees increasing interior detail as vision range allows
   - Entering mech door smoothly transitions from exterior world view to interior floor view
   - Exiting mech maintains visual continuity with world outside

### Rendering Performance
- **Selective rendering**: Only render floor tiles within player's vision range
- **Cached floor data**: Client stores all mech floors but only renders relevant ones
- **Viewport culling**: Use existing tile visibility calculations for mech interiors

## Implementation Notes

### HashMap-Based Floor Storage

- **Flexibility**: Supports irregular mech shapes, future customization, and various sizes beyond 10x10
- **Memory efficiency**: Only stores tiles that differ from default (likely mostly empty space)
- **Scalability**: Easy to expand to larger or more complex mech layouts
- **Performance**: Fast lookups for tile queries during rendering and collision detection

### Multi-Tile Station System

- **Station Definition**: Stations define their size (1x1, 2x2, 2x1, etc.) and all occupied tiles reference the same EntityId
- **Interaction Logic**: Player interaction with any tile of a multi-tile station activates the entire station
- **Rendering Consistency**: Multi-tile stations render as cohesive units, not separate tiles

### Floor Transition Mechanics

- **Instant Transitions**: No animation, immediate floor change when player steps on stairway tile
- **Isolation**: Players only see others on their current floor, no cross-floor visibility
- **Stairway Validation**: Server validates that stairway tiles exist and connect properly between floors

### Tile Extension Strategy

- **Stairway Tiles**: Add `Stairway(StairwayDirection, target_floor)` variant to existing Tile enum
- **Backward Compatibility**: Existing tile rendering continues to work, new variants handled separately
- **Station Tiles**: Consider whether multi-tile stations need special tile variants or use entity references

### Network Optimization

- **Initial Load**: Send complete mech interior data when player first enters mech
- **Incremental Updates**: Send only changed tiles for real-time updates (damage, repairs)
- **Caching**: Client caches floor data until player leaves mech or significant changes occur

## Validation Gates

- **Floor Data Consistency**: Each mech must have exactly 3 floors with valid HashMap tile storage
- **Stairway Connectivity**: All stairways must have corresponding connections between floors
- **Station Placement**: Multi-tile stations must fit within floor boundaries without overlap
- **Network Protocol**: MechFloorData messages must serialize/deserialize correctly
- **Rendering Performance**: Floor rendering must not impact game performance (target: 60 FPS)
- **Player Visibility**: Players must only see others on the same floor
- **Floor Transitions**: Stairway interactions must instantly and reliably change floors

## Implementation Checkpoints/Testing

### 1. Data Structure Implementation

- Implement MechFloor, MechInterior, and position types in shared crate
- Add MechFloorData and FloorTransition message types
- Test serialization/deserialization of new data structures
- Command to verify: `cargo test --package shared floor_data`

### 2. Basic Floor Generation

- Implement simple procedural floor generation with stations and stairways
- Generate test mechs with 3 floors containing different station layouts
- Verify floor connectivity and stairway placement
- Command to verify: `cargo test --package server mech_generation`

### 3. Server Floor Management

- Implement floor transition handling in game logic
- Add player-to-floor tracking in mech data
- Handle MechFloorData message sending to clients
- Command to verify: `cargo test --package server floor_transition`

### 4. Client Floor Rendering

- Implement floor data management and caching
- Add floor-specific rendering in client
- Handle floor transition messages and UI updates
- Command to verify: `just build-web` and test in browser

### 5. Multi-Tile Station System

- Implement multi-tile station placement and rendering
- Add station interaction logic for larger stations
- Test various station sizes (1x1, 2x2, 2x1)
- Command to verify: `just test-multiplayer` and verify station interactions

### 6. Integration Testing

- Test complete flow: enter mech → see floor → use stairway → see different floor
- Verify multiplayer isolation per floor
- Test network message performance with multiple players
- Command to verify: `just dev` and test with multiple browser tabs

## Other Considerations

- **Memory Usage**: HashMap storage is more efficient than fixed grids, but monitor memory usage with many mechs and complex floors
- **Network Bandwidth**: Sending complete floor data could be expensive; consider compression
- **Future Scalability**: Architecture should support different floor sizes and mech customization
- **Performance**: Rendering 10×10 floors should be optimized, possibly with viewport culling
- **Backward Compatibility**: Ensure existing mech systems continue working during transition
- **Station Complexity**: Multi-tile stations may need more sophisticated entity management
- **Error Handling**: Floor transitions must handle edge cases (invalid floors, missing stairways)
- **Data Persistence**: Consider whether mech floor layouts should persist across server restarts

---

## Template Usage Examples

### For Server Logic
- Goal: "Handle player floor transitions via stairway tiles"
- Files to Reference: "server/src/game.rs for player movement patterns"
- Validation Gates: "Floor transitions must be instant and update player visibility correctly"

### For Client Rendering
- Goal: "Render detailed 10x10 mech floors with multi-tile stations"
- Files to Reference: "client/src/main.rs for existing rendering patterns"
- Implementation Notes: "Must maintain 60 FPS performance with detailed floor rendering"

### For Data Structures
- Goal: "Define MechInterior and floor layout data types"
- Files to Reference: "shared/src/types.rs for serialization patterns"
- Validation Gates: "All data types must serialize correctly for network transmission"
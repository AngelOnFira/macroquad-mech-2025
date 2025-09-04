name: "Mech Spatial Positioning System Enhancement"
description: |
  Remove camera transition systems and implement true spatial positioning where entering a mech 
  feels like walking into any physical space - with walls naturally occluding the outside world 
  rather than switching views or cameras.

---

## Goal

**REMOVE** all camera transition and view-switching systems. Implement true spatial positioning where entering a mech is exactly like walking into a building - the camera stays in world space continuously, and walls naturally occlude what you can't see. No transitions, no camera changes, just pure spatial movement with natural occlusion.

## Why

- **True Spatial Immersion**: Players should experience entering a mech exactly like walking into a building - natural, seamless, with no camera changes or view switches
- **Eliminate Jarring Transitions**: Remove all transition systems that break the spatial illusion and make entry feel like a mode switch
- **Natural Occlusion**: Walls should naturally block vision of the outside world when inside, just like in real physical spaces
- **Continuous World Space**: Camera and rendering stay in unified world coordinates - no coordinate system switching
- **Foundation for Physics**: True spatial positioning enables realistic physics, lighting, and environmental effects across boundaries
- **Multiplayer Clarity**: All players see the same continuous spatial relationships without view-dependent transitions

## What

The current system uses view switching - rendering different views based on whether the player is inside or outside a mech. This creates a jarring transition that breaks spatial immersion. Instead, we need to **REMOVE** all transition systems and implement unified spatial rendering.

### Core Components to REMOVE:

- **All Transition Systems**: Delete TransitionState, TransitionType, and smooth_transitions modules
- **View Switching Logic**: Remove the switch between world view and interior view rendering  
- **Camera Mode Changes**: Eliminate any camera coordinate system switching
- **Entry/Exit Animations**: Remove smooth transition animations that treat entry as a mode change

### Core Components to IMPLEMENT:

- **Unified World Rendering**: Always render everything in world space - both exteriors and interiors simultaneously
- **Natural Occlusion**: When inside a mech, walls naturally hide the outside world (not camera changes)
- **Continuous Vision**: Vision system handles what's visible through doors/windows without mode switches
- **Spatial Testing**: Verify the unified system works correctly with moving mechs

### Scope Boundaries:

- **In Scope**: Remove transitions, implement unified rendering, natural occlusion, continuous spatial experience
- **Out of Scope**: Networking protocol changes (coordinates system already supports this), major architectural overhaul

## Endpoints/APIs to Implement

N/A - This is primarily a client-side rendering and game logic enhancement using existing network protocol.

## Current Directory Structure

```
mech-battle-arena/
├── shared/src/
│   ├── mech_coordinates.rs        # Advanced coordinate transformation system
│   ├── mech_layout.rs            # Mech interior layout generation
│   ├── coordinates.rs            # Unified coordinate types and conversions
│   ├── types.rs                 # PlayerLocation enum with InsideMech support
│   ├── vision.rs               # Vision system with mech interior support
│   └── tile_entity.rs          # Hybrid tile-entity system
├── server/src/
│   ├── commands.rs             # Player movement and mech entry logic
│   ├── game.rs                # Main game state management
│   └── systems/
│       └── physics.rs         # Mech movement and collision system
├── client/src/
│   ├── rendering/
│   │   ├── mech_interior.rs   # Mech interior rendering with world mapping
│   │   └── world.rs          # World rendering system
│   └── game_state.rs         # Client game state management
└── docs/
    └── HYBRID_TILE_ENTITY_SYSTEM.md  # System documentation
```

## Proposed Directory Structure

```
mech-battle-arena/
├── shared/src/
│   ├── mech_coordinates.rs        # [ENHANCED] Add debug visualization helpers
│   ├── mech_layout.rs            # [EXISTING] Interior layout generation
│   ├── coordinates.rs            # [EXISTING] Coordinate system
│   ├── types.rs                 # [EXISTING] PlayerLocation types
│   ├── vision.rs               # [EXISTING] Vision system
│   └── tile_entity.rs          # [EXISTING] Hybrid tile system
├── server/src/
│   ├── commands.rs             # [ENHANCED] Improve mech entry smoothness
│   ├── game.rs                # [ENHANCED] Add testing mode for slow mech movement
│   └── systems/
│       └── physics.rs         # [ENHANCED] Add configurable mech movement speeds
├── client/src/
│   ├── rendering/
│   │   ├── unified_world.rs  # [NEW] Unified rendering system for all world space
│   │   ├── occlusion.rs     # [NEW] Natural occlusion system for mech walls
│   │   └── spatial_debug.rs  # [EXISTING] Debug visualizations for coordinate transforms
│   ├── game_state.rs         # [MODIFIED] Remove transition state and related code
│   └── spatial_testing.rs   # [EXISTING] Testing utilities for spatial positioning
└── docs/
    ├── HYBRID_TILE_ENTITY_SYSTEM.md  # [EXISTING] System documentation
    └── SPATIAL_POSITIONING_TESTING.md # [NEW] Testing guide for spatial features
```

## Files to Reference

- `/home/forest/Documents/git/mech-battle-arena/shared/src/mech_coordinates.rs` (read_only) Core coordinate transformation system - already implements world-to-interior mapping
- `/home/forest/Documents/git/mech-battle-arena/server/src/commands.rs` (read_only) Current mech entry logic in PlayerInputCommand::execute - lines 178-261 show the entry mechanism
- `/home/forest/Documents/git/mech-battle-arena/client/src/rendering/mech_interior.rs` (read_only) Current interior rendering - already uses world coordinate mapping
- `/home/forest/Documents/git/mech-battle-arena/server/src/systems/physics.rs` (read_only) Mech movement system - lines 20-63 show how mechs move in world space
- `/home/forest/Documents/git/mech-battle-arena/shared/src/types.rs` (read_only) PlayerLocation enum - already supports InsideMech with mech_id, floor, and relative position
- `/home/forest/Documents/git/mech-battle-arena/docs/HYBRID_TILE_ENTITY_SYSTEM.md` (read_only) Architecture documentation for the tile-entity system
- Web research on relative coordinate systems: https://gamedev.stackexchange.com/questions/193983/how-do-game-engines-enforce-global-engine-specific-coordinate-systems
- Web research on multiplayer synchronization: https://medium.com/@qingweilim/how-do-multiplayer-games-sync-their-state-part-1-ab72d6a54043

## Files to Implement (concept)

### Systems to REMOVE

1. **Delete `client/src/rendering/smooth_transitions.rs`** - Remove all transition animation code
2. **Remove from `client/src/game_state.rs`** - Delete TransitionState struct and TransitionType enum  
3. **Modify `client/src/rendering/mod.rs`** - Remove render_transition method and view switching logic

### Systems to IMPLEMENT

1. `client/src/rendering/unified_world.rs` - Unified rendering system

```rust
use crate::game_state::GameState;
use crate::vision::ClientVisionSystem;
use shared::{MechInteriorCoordinates, WorldPos, TilePos, PlayerLocation};

pub struct UnifiedWorldRenderer;

impl UnifiedWorldRenderer {
    /// Render everything in world space - no view switching
    pub fn render_unified_world(
        game_state: &GameState, 
        cam_x: f32, 
        cam_y: f32, 
        vision_system: Option<&ClientVisionSystem>
    ) {
        // ALWAYS render world tiles first (base layer)
        Self::render_world_layer(game_state, cam_x, cam_y, vision_system);
        
        // ALWAYS render all mech interiors in their world positions
        for mech in game_state.mechs.values() {
            Self::render_mech_interior_in_world_space(mech, game_state, cam_x, cam_y, vision_system);
        }
        
        // Render entities (players, resources, etc.) in their world positions
        Self::render_entities_in_world_space(game_state, cam_x, cam_y, vision_system);
    }
    
    fn render_world_layer(game_state: &GameState, cam_x: f32, cam_y: f32, vision: Option<&ClientVisionSystem>) {
        // Render outdoor world tiles - always visible unless occluded
        super::world::render_world_tiles(game_state, cam_x, cam_y, vision);
    }
    
    fn render_mech_interior_in_world_space(
        mech: &MechState,
        game_state: &GameState, 
        cam_x: f32, 
        cam_y: f32,
        vision: Option<&ClientVisionSystem>
    ) {
        // Render all floors of this mech in world space using coordinate transformation
        for floor in 0..shared::MECH_FLOORS as u8 {
            // Use MechInteriorCoordinates to map interior tiles to world positions
            super::mech_interior::render_mech_floor_in_world_space(
                mech, floor, game_state, cam_x, cam_y, vision
            );
        }
    }
}
```

2. `client/src/rendering/occlusion.rs` - Natural occlusion system

```rust
use crate::game_state::GameState;
use shared::{PlayerLocation, MechInteriorCoordinates, TilePos};

pub struct OcclusionSystem;

impl OcclusionSystem {
    /// Determine what should be occluded based on player position and mech walls
    pub fn calculate_occlusion(game_state: &GameState) -> OcclusionMask {
        match &game_state.player_location {
            PlayerLocation::OutsideWorld(pos) => {
                // Outside: can see world + interiors through doors/windows
                Self::calculate_exterior_occlusion(*pos, game_state)
            }
            PlayerLocation::InsideMech { mech_id, floor, pos } => {
                // Inside: mech walls occlude outside world naturally  
                Self::calculate_interior_occlusion(*mech_id, *floor, *pos, game_state)
            }
        }
    }
    
    fn calculate_interior_occlusion(
        mech_id: Uuid,
        floor: u8,
        interior_pos: WorldPos,
        game_state: &GameState
    ) -> OcclusionMask {
        let mut mask = OcclusionMask::new();
        
        // When inside, mech walls naturally block view of outside world
        if let Some(mech) = game_state.mechs.get(&mech_id) {
            // Calculate which exterior tiles are blocked by mech walls
            // Only doors and windows allow visibility to outside
            mask.occlude_exterior_except_openings(mech.position, floor, interior_pos);
        }
        
        mask
    }
    
    fn calculate_exterior_occlusion(pos: WorldPos, game_state: &GameState) -> OcclusionMask {
        let mut mask = OcclusionMask::new();
        
        // From outside, can see into mechs through doors/windows
        // Mech walls block view of interior tiles behind them
        for mech in game_state.mechs.values() {
            mask.calculate_mech_interior_visibility(pos, mech.position);
        }
        
        mask
    }
}

pub struct OcclusionMask {
    // Spatial mask indicating what tiles are occluded
    occluded_tiles: std::collections::HashSet<TilePos>,
}
```

### Server-Side Testing Enhancements

3. `server/src/testing_modes.rs` - Testing configuration system

```rust
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TestingConfig {
    pub slow_mech_movement: bool,
    pub mech_movement_speed: f32,  // Override normal speed
    pub mech_movement_direction: (f32, f32),  // Normalized direction vector
    pub enable_coordinate_logging: bool,
    pub spatial_debug_mode: bool,
}

impl TestingConfig {
    pub fn create_spatial_test_config() -> Self {
        Self {
            slow_mech_movement: true,
            mech_movement_speed: 0.3,  // tiles per second (very slow)
            mech_movement_direction: (0.0, 1.0),  // South
            enable_coordinate_logging: true,
            spatial_debug_mode: true,
        }
    }
}

pub struct TestingManager {
    config: TestingConfig,
    mech_test_overrides: HashMap<Uuid, (f32, f32)>,  // Per-mech velocity overrides
}

impl TestingManager {
    pub fn apply_mech_movement_overrides(&self, mech_velocities: &mut HashMap<Uuid, (f32, f32)>) {
        if self.config.slow_mech_movement {
            for (mech_id, velocity) in mech_velocities.iter_mut() {
                if self.mech_test_overrides.contains_key(mech_id) {
                    *velocity = self.mech_test_overrides[mech_id];
                } else {
                    *velocity = (
                        self.config.mech_movement_direction.0 * self.config.mech_movement_speed,
                        self.config.mech_movement_direction.1 * self.config.mech_movement_speed,
                    );
                }
            }
        }
    }
}
```

### Testing Utilities

4. `client/src/spatial_testing.rs` - Client-side testing utilities

```rust
use shared::{MechInteriorCoordinates, PlayerLocation, WorldPos, TilePos};

pub struct SpatialTestSuite {
    test_results: Vec<TestResult>,
    current_test: Option<RunningTest>,
}

#[derive(Debug)]
pub struct TestResult {
    test_name: String,
    success: bool,
    details: String,
    timestamp: std::time::Instant,
}

#[derive(Debug)]
pub struct RunningTest {
    name: String,
    start_time: std::time::Instant,
    expected_behavior: String,
    measurements: Vec<SpatialMeasurement>,
}

#[derive(Debug, Clone)]
pub struct SpatialMeasurement {
    timestamp: f64,
    player_location: PlayerLocation,
    mech_world_position: WorldPos,
    calculated_world_position: WorldPos,  // What the coordinate system thinks
    actual_render_position: WorldPos,     // Where we actually rendered
}

impl SpatialTestSuite {
    pub fn start_mech_entry_test(&mut self, mech_id: Uuid, entry_door: TilePos) {
        // Begin test to verify smooth spatial entry
        self.current_test = Some(RunningTest {
            name: "Mech Entry Spatial Continuity".to_string(),
            start_time: std::time::Instant::now(),
            expected_behavior: "Player should smoothly transition from world to interior coordinates".to_string(),
            measurements: Vec::new(),
        });
    }
    
    pub fn record_spatial_measurement(&mut self, player_location: PlayerLocation, mech_world_pos: WorldPos) {
        if let Some(ref mut test) = self.current_test {
            // Calculate what the coordinate system thinks the position should be
            let calculated_pos = match &player_location {
                PlayerLocation::OutsideWorld(pos) => *pos,
                PlayerLocation::InsideMech { mech_id, floor, pos } => {
                    // Use coordinate transformation to get world position
                    let interior_tile = pos.to_tile();
                    let world_tile = MechInteriorCoordinates::interior_to_world(
                        mech_world_pos.to_tile(), *floor, interior_tile
                    );
                    world_tile.to_world_center()
                }
            };
            
            test.measurements.push(SpatialMeasurement {
                timestamp: test.start_time.elapsed().as_secs_f64(),
                player_location,
                mech_world_position: mech_world_pos,
                calculated_world_position: calculated_pos,
                actual_render_position: calculated_pos, // Will be updated by renderer
            });
        }
    }
    
    pub fn verify_relative_movement_in_moving_mech(&self) -> TestResult {
        // Analyze measurements to verify that:
        // 1. Player interior movement is independent of mech movement
        // 2. Player world position moves with mech when inside
        // 3. Coordinate transformations remain consistent
        
        TestResult {
            test_name: "Relative Movement in Moving Mech".to_string(),
            success: true, // Will be calculated based on measurements
            details: "Player movement relative to mech interior maintained while mech moves".to_string(),
            timestamp: std::time::Instant::now(),
        }
    }
}
```

## Implementation Notes

### Understanding Current System

The codebase analysis reveals that the desired functionality **already exists** in the current implementation:

1. **Coordinate Transformation**: `MechInteriorCoordinates::interior_to_world()` already maps interior positions to world space
2. **Spatial Movement**: Players already move within mech bounds while mechs move in world space
3. **Entry Mechanism**: Door tiles trigger immediate coordinate space transitions
4. **Rendering**: Mech interiors already render using world coordinate mapping

### Key Enhancement Areas

The perceived "view transition" issue likely stems from:

1. **Abrupt Entry**: No smooth animation showing spatial movement through doors
2. **Visual Discontinuity**: Players may not see the spatial relationship clearly
3. **Lack of Debug Visualization**: No visual feedback showing coordinate transformations
4. **Testing Gaps**: No systematic way to verify the spatial positioning works correctly

### Coordinate System Integration

The current system uses a sophisticated approach:

```rust
// Floor mapping with Y-axis offsets for different floors
let virtual_y_offset = floor as i32 * (FLOOR_HEIGHT_TILES + 1);
TilePos::new(base_x, base_y + virtual_y_offset)
```

This allows:
- **Same XY space**: All floors exist at same world X coordinates
- **Z-level separation**: Different floors use Y-axis virtual offsets
- **Vision system**: Outside players can see into mechs through windows/doors
- **Collision detection**: Proper spatial relationships maintained

### Testing Strategy

The implementation should focus on making the existing spatial system more apparent:

1. **Visual Continuity**: Ensure players see the door → interior spatial relationship
2. **Smooth Transitions**: Animate the coordinate transformation process
3. **Debug Visualization**: Show coordinate grids, transformation vectors, mech bounds
4. **Movement Verification**: Slow mech movement to clearly show relative positioning works

### Network Synchronization Considerations

Based on multiplayer research, the system should maintain:

1. **Server Authority**: Server continues to be authoritative for positions
2. **Client Prediction**: Client can predict movement within mech bounds
3. **Lag Compensation**: Existing lag compensation continues to work
4. **State Synchronization**: PlayerLocation enum already handles both coordinate spaces

## Validation Gates

- **No View Switching**: Completely eliminate any camera or view mode changes when entering/exiting mechs
- **Natural Spatial Entry**: Walking into a mech feels exactly like walking into a building - walls naturally occlude the outside world
- **Continuous World Rendering**: Everything always renders in unified world space - no coordinate system switching
- **Natural Occlusion**: When inside, mech walls block vision of outside world through natural occlusion, not view changes
- **Moving Mech Test**: With mechs moving slowly southward, players inside experience natural spatial movement with the mech
- **Zero Transitions**: No transition states, progress bars, or animation systems involved in mech entry
- **Multiplayer Spatial Consistency**: All players see the same continuous spatial world without view-dependent rendering

## Implementation Checkpoints/Testing

### 1. Remove All Transition Systems

- **Implementation**: Delete TransitionState, smooth_transitions module, render_transition method
- **Testing**: Verify no references to transition systems remain in codebase
- **Expected Results**: Clean codebase with no transition-related code
- **Verification Command**: `grep -r "Transition\|smooth_transition" client/src/` should return no results

### 2. Implement Unified World Rendering

- **Implementation**: Replace view switching with unified rendering that always renders everything in world space
- **Testing**: Player can see both world and mech interiors simultaneously based on position and occlusion
- **Expected Results**: No mode switches - everything renders in continuous world space
- **Verification Command**: Enter/exit mechs and verify camera never changes coordinate systems

### 3. Natural Occlusion System

- **Implementation**: Mech walls naturally hide outside world when inside, doors/windows show glimpses outside
- **Testing**: When inside mech, outside world is occluded by walls, visible through openings
- **Expected Results**: Natural spatial occlusion like a real building - no artificial view blocking
- **Verification Command**: Move around inside mech and verify occlusion feels natural

### 4. Slow Mech Movement Testing

- **Implementation**: Test with slowly moving mechs to verify spatial continuity
- **Testing**: Inside moving mech, player moves independently while naturally moving with mech
- **Expected Results**: Natural spatial movement - like being inside a moving vehicle
- **Verification Command**: `cargo run --bin server -- --testing-mode slow-mech-south`

### 5. Zero Transition Verification

- **Implementation**: Ensure absolutely no camera changes, view switches, or transition states
- **Testing**: Walk in and out of mechs repeatedly, verify zero jarring changes
- **Expected Results**: Entry/exit feels like walking through any doorway - completely natural
- **Verification Command**: Test entry/exit extensively and confirm zero transition artifacts

## Other Considerations

### Performance Implications

- **Coordinate Transformation Overhead**: The existing `MechInteriorCoordinates` system is already optimized with caching
- **Rendering Performance**: World coordinate mapping for interior rendering is already implemented efficiently  
- **Network Traffic**: No changes to network protocol required, existing `PlayerLocation` enum handles both coordinate spaces

### Backward Compatibility

- **Existing Save Data**: No breaking changes to game state or save formats
- **Network Protocol**: All changes are client-side rendering and server-side testing enhancements
- **API Compatibility**: No changes to public APIs or message formats

### Security Concerns

- **Movement Validation**: Server continues to validate all player movements in both coordinate spaces
- **Coordinate Bounds Checking**: Existing bounds checking for both world and interior coordinates maintained
- **Anti-Cheat**: No new attack vectors introduced, coordinate transformations remain server-authoritative

### Future Extensibility

- **Physics Integration**: True spatial positioning enables future physics interactions across mech boundaries
- **Advanced Vision**: Enhanced line-of-sight calculations between interior and exterior spaces
- **Environmental Effects**: Atmospheric effects, lighting, and particle systems that work across coordinate spaces
- **Modding Support**: Clear coordinate transformation APIs enable mods to work with both coordinate spaces

### Dependencies

- **Existing Crate Dependencies**: No new external dependencies required
- **Internal Dependencies**: Builds on existing coordinate system, vision system, and hybrid tile-entity architecture
- **Tool Dependencies**: Uses existing development tools (just, cargo, etc.)

### Potential Risks and Limitations

- **User Expectation Management**: Users may expect more dramatic changes than the subtle enhancements this PRP provides
- **Coordinate System Complexity**: The existing virtual Y-offset system may be confusing to new developers
- **Debug Visualization Overhead**: Debug rendering features should be disabled in production builds
- **Testing Mode Performance**: Slow mech movement testing should only be enabled in development/testing environments

---

## Research Foundation

This PRP is based on comprehensive analysis of:

1. **Existing Codebase Architecture**: Deep examination of coordinate systems, player movement, mech entry logic, and rendering systems
2. **Game Development Best Practices**: Research into hierarchical transform systems, relative coordinate spaces, and moving platform implementations
3. **Multiplayer Networking Patterns**: Analysis of client-server synchronization for entities on moving platforms
4. **Hybrid Tile-Entity System Documentation**: Understanding of the current tile-based world representation and entity component system integration

The implementation leverages the sophisticated existing architecture while enhancing user experience through improved visualization and testing capabilities.
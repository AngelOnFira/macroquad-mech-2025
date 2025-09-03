# Mech Battle Arena - Development Guide

> Think carefully and implement the most concise solution that changes as little code as possible.

## USE SUB-AGENTS FOR CONTEXT OPTIMIZATION

### 1. Always use the file-analyzer sub-agent when asked to read files.
The file-analyzer agent is an expert in extracting and summarizing critical information from files, particularly log files and verbose outputs. It provides concise, actionable summaries that preserve essential information while dramatically reducing context usage.

### 2. Always use the code-analyzer sub-agent when asked to search code, analyze code, research bugs, or trace logic flow.

The code-analyzer agent is an expert in code analysis, logic tracing, and vulnerability detection. It provides concise, actionable summaries that preserve essential information while dramatically reducing context usage.

### 3. Always use the test-runner sub-agent to run tests and analyze the test results.

Using the test-runner agent ensures:

- Full test output is captured for debugging
- Main conversation stays clean and focused
- Context usage is optimized
- All issues are properly surfaced
- No approval dialogs interrupt the workflow

## Philosophy

### Error Handling

- **Fail fast** for critical configuration (missing text model)
- **Log and continue** for optional features (extraction model)
- **Graceful degradation** when external services unavailable
- **User-friendly messages** through resilience layer

### Testing

- Always use the test-runner agent to execute tests.
- Do not use mock services for anything ever.
- Do not move on to the next test until the current test is complete.
- If the test fails, consider checking if the test is structured correctly before deciding we need to refactor the codebase.
- Tests to be verbose so we can use them for debugging.

## Tone and Behavior

- Criticism is welcome. Please tell me when I am wrong or mistaken, or even when you think I might be wrong or mistaken.
- Please tell me if there is a better approach than the one I am taking.
- Please tell me if there is a relevant standard or convention that I appear to be unaware of.
- Be skeptical.
- Be concise.
- Short summaries are OK, but don't give an extended breakdown unless we are working through the details of a plan.
- Do not flatter, and do not give compliments unless I am specifically asking for your judgement.
- Occasional pleasantries are fine.
- Feel free to ask many questions. If you are in doubt of my intent, don't guess. Ask.

## ABSOLUTE RULES:

- NO PARTIAL IMPLEMENTATION
- NO SIMPLIFICATION : no "//This is simplified stuff for now, complete implementation would blablabla"
- NO CODE DUPLICATION : check existing codebase to reuse functions and constants Read files before writing new functions. Use common sense function name to find them easily.
- NO DEAD CODE : either use or delete from codebase completely
- IMPLEMENT TEST FOR EVERY FUNCTIONS
- NO CHEATER TESTS : test must be accurate, reflect real usage and be designed to reveal flaws. No useless tests! Design tests to be verbose so we can use them for debuging.
- NO INCONSISTENT NAMING - read existing codebase naming patterns.
- NO OVER-ENGINEERING - Don't add unnecessary abstractions, factory patterns, or middleware when simple functions would work. Don't think "enterprise" when you need "working"
- NO MIXED CONCERNS - Don't put validation logic inside API handlers, database queries inside UI components, etc. instead of proper separation
- NO RESOURCE LEAKS - Don't forget to close database connections, clear timeouts, remove event listeners, or clean up file handles

This guide helps Claude Code understand and work with the Mech Battle Arena project.

## Project Overview

A multiplayer web game where teams control giant mechs from the inside. Players can:
- Move around the world and inside mechs
- Operate various stations (weapons, shields, engines, etc.)
- Collect resources to upgrade their mechs
- Engage in team vs team combat

## Quick Start

### Using DevTabs (Recommended)
The project includes a `devtabs.yaml` configuration that manages all development processes:

```bash
# DevTabs will automatically start:
# - game-server: The Rust game server
# - web-server: Python HTTP server for WASM
# - build-wasm: Builds the WebAssembly client

# Just run DevTabs and it handles everything
```

### Manual Setup (Alternative)
```bash
# Or use Just commands:
just dev            # Start everything manually

# Or step by step:
just build-web      # Build WebAssembly client
just server         # Start game server (in one terminal)
just web-server     # Start web server (in another terminal)
```

Then open http://localhost:8080 in multiple browser tabs to test multiplayer.

### Hybrid Tile System Demo
```bash
# Run the interactive demo showcasing the hybrid tile-entity system
just dev-demo
# Then open http://localhost:8080/demo.html

# Demo features:
# - Layered floor system (1/2/3 keys to switch)
# - Raycasting vision (V to toggle)
# - Window mechanics
# - Station entities
# - Continuous movement (WASD)
```

## Common Development Tasks

### Using Just Commands

```bash
just --list              # Show all available commands
just dev                 # Start full development environment
just build               # Build all components
just test-multiplayer    # Test with two native clients
just kill-servers        # Stop all servers
just check              # Run lints and checks
just clean-all          # Clean everything including logs
```

### Building

```bash
just build-server       # Build game server
just build-client       # Build native client
just build-web         # Build WebAssembly client
just release           # Build all release versions
```

### Testing

```bash
just test              # Run unit tests
just test-client Bob   # Run test client with name "Bob"
just test-multiplayer  # Run two test clients
```

### Browser Testing with Playwright
The game includes Playwright tests to verify the web version loads and functions correctly:

```bash
# Run Playwright tests
npm test               # or yarn test / pnpm test

# Tests verify:
# - Game loads in browser
# - WebSocket connection establishes
# - Canvas renders properly
# - Basic game functionality
```

## Architecture

### Workspace Structure
- `server/` - Axum WebSocket server, authoritative game state
- `client/` - Macroquad game client (native and WASM)
- `shared/` - Protocol definitions and shared types
- `docs/` - Design documents and architecture guides

### Key Technologies
- **Networking**: WebSockets (native: `ws`, web: `web-sys`)
- **Graphics**: Macroquad (works in both native and WASM)
- **Serialization**: JSON via serde_json
- **Server**: Axum with tokio async runtime
- **Tile System**: Hybrid tile-entity approach (see below)

### Network Protocol
All messages are JSON-serialized enums defined in `shared/src/messages.rs`:
- `ClientMessage`: Player inputs, join requests, station controls
- `ServerMessage`: State updates, events, confirmations

### Hybrid Tile-Entity System
The game uses a hybrid approach for world representation:
- **Static Tiles**: Simple enums for walls, floors, windows (90% of tiles)
- **Entity References**: Complex objects (stations, turrets) use ECS with UUID references
- **Benefits**: Fast performance for simple tiles, flexibility for complex objects
- See `docs/HYBRID_TILE_ENTITY_SYSTEM.md` for full details

## Development Workflow

1. **Start Development Environment**: DevTabs handles all processes automatically
2. **Make Changes**: Edit code in appropriate crate
3. **Test Locally**: 
   - Open http://localhost:8080 in browser tabs
   - DevTabs auto-rebuilds WASM on changes
4. **Run Tests**:
   ```bash
   just test           # Unit tests
   npm test           # Playwright browser tests
   ```
5. **Check Quality**: 
   ```bash
   just check         # Lints and type checks
   ```

## Important Files

- `devtabs.yaml` - Development environment configuration
- `justfile` - All development commands
- `shared/src/messages.rs` - Network protocol
- `shared/src/types.rs` - Game types
- `shared/src/tile_entity.rs` - Hybrid tile system core
- `shared/src/vision.rs` - Raycasting vision system
- `shared/src/components.rs` - ECS components for entities
- `server/src/game.rs` - Core game logic
- `server/src/entity_storage.rs` - Entity management system
- `client/src/main.rs` - Client entry point
- `client/src/network_web.rs` - WebSocket for browsers
- `client/src/bin/demo.rs` - Interactive hybrid system demo
- `docs/HYBRID_TILE_ENTITY_SYSTEM.md` - Tile system documentation
- `docs/MECH_INTERIOR_DESIGN.md` - Mech interior implementation

## Debugging Tips

### Server Issues
```bash
just kill-servers      # Kill stuck servers
just check-ports      # See what's using ports
tail -f server.log    # Watch server logs
```

### WASM Build Issues
- Ensure `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Check browser console for errors (F12)
- Verify WebSocket URL matches server

### Multiplayer Testing
- Each player gets unique ID and random name
- Teams auto-balance (red vs blue)
- Use multiple browser tabs or `just test-multiplayer`

## Key Game Mechanics

- **Movement**: WASD keys, continuous movement over tiles
- **Mechs**: 3 floors (layered Z-levels), multiple stations, team-owned
- **Vision**: Raycasting with line-of-sight, dark interiors, window mechanics
- **Resources**: 4 types, used for upgrades/repairs
- **Combat**: Laser (instant) and projectile weapons
- **Upgrades**: Improve weapons, shields, engines
- **Tiles**: Hybrid system with static tiles and entity-based objects

## WebAssembly Specifics

The client uses conditional compilation:
- `#[cfg(target_arch = "wasm32")]` - Web-specific code
- `#[cfg(not(target_arch = "wasm32"))]` - Native-specific code

Key differences:
- Web uses `web-sys` WebSocket, native uses `ws` crate
- Web connects to page origin, native to localhost
- Web uses console logging, native uses env_logger

### ⚠️ Important WASM Dependency Constraint

**Macroquad is incompatible with `wasm-bindgen`-based dependencies.**

When adding new dependencies for web features, avoid crates that depend on `wasm-bindgen`. This includes:
- `egui_plot` - Uses wasm-bindgen internally
- Many web-specific utility crates
- Direct `wasm-bindgen` or `web-sys` usage in dependencies

**Why**: Macroquad uses its own WebAssembly loader and build system that conflicts with wasm-bindgen's approach.

**Alternatives**:
- Use macroquad's built-in features when possible
- Implement simple custom solutions (like ASCII charts instead of complex plots)
- Check if pure Rust alternatives exist
- Consider feature flags to disable problematic dependencies in WASM builds

**Current known incompatible dependencies that were removed**:
- `egui_plot` - Replaced with simple ASCII charts and text-based metrics

## Tile Math and Rendering Abstractions

**Use these abstractions instead of manual calculations to ensure consistency and maintainability.**

### Coordinate Conversions
- Use `WorldPos`, `TilePos`, `ScreenPos` from `shared::coordinates`
- **Never manually multiply by `TILE_SIZE`** - use conversion methods instead
- Use `to_world()`, `to_tile()`, `to_world_center()` for conversions

### Mech Door Positioning
```rust
// ❌ Before: Manual door calculation
let door_x1 = mech.position.x + (MECH_SIZE_TILES / 2) - 1;
let door_x2 = mech.position.x + (MECH_SIZE_TILES / 2);
let entry_x = if tile_pos.x == door_x1 {
    (FLOOR_WIDTH_TILES as f32 / 2.0 - 0.5) * TILE_SIZE
} else if tile_pos.x == door_x2 {
    (FLOOR_WIDTH_TILES as f32 / 2.0 + 0.5) * TILE_SIZE
} else {
    (FLOOR_WIDTH_TILES as f32 / 2.0) * TILE_SIZE
};

// ✅ After: Use MechDoorPositions abstraction
use shared::coordinates::MechDoorPositions;
let doors = MechDoorPositions::from_mech_position(mech.position);
let entry_pos = doors.get_entry_position(tile_pos);
```

### Arrow Drawing
```rust
// ❌ Before: Manual triangle drawing (20+ lines of repetitive code)
let center_x = x + size / 2.0;
let center_y = y + size / 2.0;
let arrow_size = size * 0.3;
match facing {
    Direction::Up => {
        draw_triangle(
            Vec2::new(center_x, center_y - arrow_size),
            Vec2::new(center_x - arrow_size/2.0, center_y),
            Vec2::new(center_x + arrow_size/2.0, center_y),
            arrow_color,
        );
    }
    // ... 15 more lines for other directions
}

// ✅ After: Use ArrowRenderer primitives
use crate::rendering::primitives::{ArrowRenderer, ArrowStyle};
ArrowRenderer::draw_arrow_centered_in_tile(
    tile_pos,
    Direction::Up,
    ArrowStyle::default().with_color(arrow_color)
);
```

### Tile Highlighting and UI Elements
```rust
// ✅ Use rendering primitives for consistent styling
use crate::rendering::primitives::{TileHighlight, TileHighlightStyle};

// Highlight selected tiles
TileHighlight::draw_tile(selected_tile, camera_offset, TileHighlightStyle::selection());

// Highlight hovered tiles
TileHighlight::draw_tile(hovered_tile, camera_offset, TileHighlightStyle::hover());

// Highlight dangerous areas
TileHighlight::draw_tiles(&danger_tiles, camera_offset, TileHighlightStyle::danger());
```

### Camera Operations
```rust
// ✅ Use Camera utilities instead of manual offset calculations
use crate::rendering::camera::{Camera, ViewportCalculations, ScreenSpace};

let mut camera = Camera::new(WorldPos::new(100.0, 200.0));

// Get visible tiles for rendering optimization
let visible_tiles = ViewportCalculations::get_visible_tile_range(&camera);

// Convert mouse to world/tile coordinates
let mouse_world = ScreenSpace::mouse_to_world(&camera);
let mouse_tile = ScreenSpace::mouse_to_tile(&camera);

// Check if something is visible before rendering
if ViewportCalculations::is_tile_visible(&camera, tile_pos) {
    render_tile(tile_pos);
}
```

### Tile Math Operations
```rust
// ✅ Use tile_math utilities for common calculations
use shared::tile_math::{TileDistance, TileNavigation, MechPositioning};

// Distance calculations
let tile_distance = TileDistance::tile_distance(pos1, pos2);
let within_range = TileDistance::within_tile_radius(player_pos, target_pos, 5.0);

// Navigation and pathfinding
let adjacent_tiles = TileNavigation::adjacent_tiles(current_tile, Some(bounds));
let line_tiles = TileNavigation::line_of_tiles(start_tile, end_tile);
let circle_tiles = TileNavigation::tiles_in_circle(center_tile, radius);

// Mech-specific calculations
let mech_center = MechPositioning::mech_center(mech_pos);
let is_inside = MechPositioning::is_inside_mech(world_pos, mech_pos);
```

### Tile Regions and Iteration
```rust
// ✅ Use TileRegion for working with areas
use shared::coordinates::{TileRegion, RelativePosition};

let region = TileRegion::new(min_tile, max_tile);
let world_bounds = TileRegion::world_bounds();
let mech_floor = TileRegion::mech_floor_bounds();

// Iterate over all tiles in a region
for tile_pos in region.iter() {
    process_tile(tile_pos);
}

// Position entities within tiles
let entity_pos = RelativePosition::Center.world_pos_in_tile(tile_pos);
let corner_pos = RelativePosition::TopLeft.world_pos_in_tile(tile_pos);
```

### When to Add New Abstractions

**Add new abstractions when you encounter these patterns:**

1. **Duplicate calculations**: If you write the same tile calculation twice, abstract it
2. **Complex rendering**: If rendering code is >10 lines for a simple shape, create a primitive
3. **Coordinate conversions**: If you need new coordinate space conversions, extend existing converters
4. **Pattern repetition**: If you see similar math patterns across files, create utilities

### Performance Considerations

- **Tile calculations are hot paths** - keep utility functions inline-friendly
- **Cache visible tile ranges** when camera hasn't moved
- **Use integer tile positions** until final render step
- **Batch similar rendering operations** using the primitives

### Common Patterns to Avoid

```rust
// ❌ Don't do manual tile math
let tile_x = (pos.x / TILE_SIZE).floor() as i32;
let world_x = tile.x as f32 * TILE_SIZE;

// ❌ Don't repeat arrow drawing logic
// (20+ lines of manual triangle calculations)

// ❌ Don't manually calculate door positions
let door_x = mech_pos.x + MECH_SIZE_TILES/2 - 1;

// ❌ Don't do manual camera transforms
let screen_x = world_pos.x + camera_offset.x;

// ✅ Use the abstractions instead!
```

### Migration Strategy

When updating existing code:
1. **Identify patterns** that match the abstractions
2. **Replace gradually** - don't break existing functionality
3. **Test thoroughly** - ensure math precision is maintained
4. **Remove old code** once abstractions are proven stable

These abstractions make the codebase more maintainable, consistent, and easier to understand. Use them whenever possible!
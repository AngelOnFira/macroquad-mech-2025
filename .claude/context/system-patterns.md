---
created: 2025-09-03T18:04:12Z
last_updated: 2025-09-03T18:04:12Z
version: 1.0
author: Claude Code PM System
---

# System Patterns & Architecture

## Core Architectural Patterns

### Hybrid Tile-Entity System
**Pattern:** Mixed static/dynamic world representation  
**Implementation:** 90% static tiles, 10% dynamic entities  
**Benefits:** Performance for simple tiles, flexibility for complex objects

```rust
// Static tiles for walls, floors, windows
enum TileType {
    Wall, Floor, Window, Grass
}

// Entity references for complex objects
struct EntityTile {
    entity_id: Uuid,
    entity_type: EntityType,
}
```

### Entity-Component-System (ECS)
**Pattern:** Compositional game architecture  
**Usage:** Complex game objects (stations, mechs, players)  
**Storage:** UUID-based entity management system

```rust
// Components define capabilities
struct Position { x: f32, y: f32 }
struct Health { current: f32, max: f32 }
struct Station { station_type: StationType }

// Entities are UUID + component collections
type EntityId = Uuid;
```

### Client-Server Architecture
**Pattern:** Authoritative server model  
**Protocol:** JSON over WebSockets  
**State:** Server maintains canonical game state

```rust
// Message-based communication
enum ClientMessage {
    Join(String),
    Move(Direction),
    UseStation(StationId),
}

enum ServerMessage {
    GameState(GameState),
    PlayerJoined(PlayerId),
    StateUpdate(StateChange),
}
```

## Data Flow Patterns

### Network Communication
**Flow:** Client Input → Server Logic → State Broadcast  
**Serialization:** Strongly-typed enums with serde  
**Reliability:** WebSocket automatic reconnection

### Game State Management
**Pattern:** Single source of truth (server)  
**Updates:** Incremental state changes  
**Synchronization:** Full state + delta updates

### Vision System
**Pattern:** Raycasting with line-of-sight  
**Implementation:** Per-player visibility calculation  
**Features:** Fog of war, window transparency

## Concurrency Patterns

### Server Concurrency
**Runtime:** Tokio async/await  
**Pattern:** Actor-like message handling  
**Connection Management:** Concurrent client handlers

```rust
// Async message handling
async fn handle_client_message(
    message: ClientMessage,
    game_state: Arc<Mutex<GameState>>,
    client_id: ClientId,
) -> Result<()>
```

### Client Rendering
**Pattern:** Game loop with frame timing  
**Concurrency:** Single-threaded with async networking  
**Platform:** Macroquad cross-platform abstraction

## Design Patterns

### Coordinate System Abstractions
**Pattern:** Type-safe coordinate conversions  
**Types:** `WorldPos`, `TilePos`, `ScreenPos`  
**Usage:** Prevents manual TILE_SIZE calculations

```rust
// Coordinate conversion abstractions
impl TilePos {
    fn to_world(self) -> WorldPos { ... }
    fn to_world_center(self) -> WorldPos { ... }
}
```

### Rendering Primitives
**Pattern:** Reusable graphics components  
**Examples:** Arrow drawing, tile highlighting  
**Benefits:** Consistent styling, reduced duplication

```rust
// Rendering abstraction example
ArrowRenderer::draw_arrow_centered_in_tile(
    tile_pos,
    Direction::Up,
    ArrowStyle::default().with_color(arrow_color)
);
```

### Configuration Management
**Pattern:** Workspace-level dependency sharing  
**Implementation:** Cargo workspace with shared deps  
**Benefits:** Version consistency, build optimization

## Error Handling Patterns

### Hierarchical Error Types
**Libraries:** `thiserror` for structured errors, `anyhow` for context  
**Pattern:** Domain-specific error types with context  
**Propagation:** `?` operator with error conversion

```rust
#[derive(thiserror::Error, Debug)]
enum GameError {
    #[error("Player {0} not found")]
    PlayerNotFound(PlayerId),
    #[error("Invalid tile position {0:?}")]
    InvalidPosition(TilePos),
}
```

### Graceful Degradation
**Network:** Reconnection handling  
**Features:** Optional systems continue on failure  
**Logging:** Structured error reporting with tracing

## Resource Management Patterns

### Memory Management
**Pattern:** Rust ownership system  
**Sharing:** `Arc<Mutex<T>>` for shared mutable state  
**Lifetime:** RAII for automatic cleanup

### Asset Loading
**Pattern:** Embedded resources  
**Web:** Assets served from `dist/` directory  
**Native:** Resources compiled into binary

## Testing Patterns

### Unit Testing
**Pattern:** Rust built-in testing framework  
**Mocking:** Manual dependency injection  
**Coverage:** Function-level test requirements

### Integration Testing
**Pattern:** Multi-crate functionality testing  
**Multiplayer:** Multiple client simulation  
**Browser:** Playwright for WASM validation

## Development Patterns

### Build Automation
**Pattern:** Just task runner  
**DevTabs:** Multi-process orchestration  
**Profiles:** Optimized debug builds

### Code Organization
**Pattern:** Feature-based module organization  
**Separation:** Clear client/server/shared boundaries  
**Documentation:** Inline docs with examples

## Performance Patterns

### Hot Path Optimization
**Tile Math:** Inline-friendly utility functions  
**Rendering:** Visible tile culling  
**Network:** Efficient JSON serialization

### Caching Strategies
**Camera:** Cache visible tile ranges  
**State:** Incremental updates over full state  
**Build:** Cargo incremental compilation

## Cross-Platform Patterns

### Conditional Compilation
**WASM:** `#[cfg(target_arch = "wasm32")]`  
**Native:** `#[cfg(not(target_arch = "wasm32"))]`  
**Features:** Platform-specific functionality

```rust
#[cfg(target_arch = "wasm32")]
use web_sys::WebSocket;

#[cfg(not(target_arch = "wasm32"))]
use ws::WebSocket;
```

### Abstraction Layers
**Networking:** Unified interface over platform APIs  
**Logging:** Console vs file-based logging  
**Input:** Cross-platform input handling

## Anti-Patterns & Constraints

### Avoided Patterns
- **Manual tile math:** Use coordinate abstractions
- **Direct WASM-bindgen:** Conflicts with Macroquad
- **Blocking I/O:** Use async/await patterns
- **Global state:** Prefer explicit state passing

### Known Limitations
- **WASM dependencies:** No wasm-bindgen crates
- **Single-threaded client:** No web worker support
- **JSON protocol:** Trade-off between simplicity and size
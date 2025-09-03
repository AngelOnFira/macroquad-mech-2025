---
created: 2025-09-03T18:04:12Z
last_updated: 2025-09-03T18:04:12Z
version: 1.0
author: Claude Code PM System
---

# Project Style Guide

## Coding Standards & Conventions

### Rust Code Style

#### Naming Conventions
- **Variables & functions:** `snake_case` (e.g., `player_position`, `calculate_distance`)
- **Types & structs:** `PascalCase` (e.g., `GameState`, `ClientMessage`) 
- **Constants:** `SCREAMING_SNAKE_CASE` (e.g., `TILE_SIZE`, `MAX_PLAYERS`)
- **Modules:** `snake_case` (e.g., `tile_entity`, `network_web`)
- **Crate names:** `kebab-case` (e.g., `mech-battle-arena`)

#### File Organization
```rust
// File header order:
// 1. Crate-level documentation
// 2. Use statements (external crates first, then internal)
// 3. Type definitions
// 4. Constants
// 5. Implementation blocks

use serde::{Deserialize, Serialize};        // External crates
use shared::types::{PlayerId, TeamColor};   // Internal modules

pub const TILE_SIZE: f32 = 32.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub position: WorldPos,
}
```

#### Error Handling Patterns
```rust
// Use thiserror for domain errors
#[derive(thiserror::Error, Debug)]
enum GameError {
    #[error("Player {0} not found")]
    PlayerNotFound(PlayerId),
    #[error("Invalid tile position {0:?}")]
    InvalidPosition(TilePos),
}

// Use anyhow for error context
fn complex_operation() -> anyhow::Result<()> {
    risky_function()
        .context("Failed during complex operation")?;
    Ok(())
}

// Use ? operator for error propagation
fn network_operation() -> Result<Response, NetworkError> {
    let data = fetch_data()?;
    let processed = process_data(data)?;
    Ok(Response::new(processed))
}
```

### Architecture Conventions

#### Coordinate System Usage
```rust
// ✅ Use type-safe coordinate conversions
use shared::coordinates::{WorldPos, TilePos, ScreenPos};

let world_pos = tile_pos.to_world_center();
let screen_pos = world_pos.to_screen(&camera);

// ❌ Avoid manual calculations
let world_x = tile.x as f32 * TILE_SIZE; // Don't do this
```

#### Entity Management
```rust
// ✅ Use UUID-based entity system
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Entity {
    id: Uuid,
    components: Vec<Component>,
}

// ✅ Type-safe entity references
type EntityId = Uuid;
type PlayerId = Uuid;
type StationId = Uuid;
```

#### Message Protocol
```rust
// ✅ Strongly-typed message enums
#[derive(Serialize, Deserialize, Debug)]
enum ClientMessage {
    Join { player_name: String },
    Move { direction: Direction },
    UseStation { station_id: StationId },
}

// ✅ Include message metadata when needed
#[derive(Serialize, Deserialize, Debug)]
struct NetworkMessage<T> {
    timestamp: u64,
    player_id: PlayerId,
    data: T,
}
```

### Documentation Standards

#### Code Documentation
```rust
/// Calculates the shortest distance between two tile positions.
/// 
/// This function uses Euclidean distance calculation and returns
/// the result as a floating-point number representing tiles.
/// 
/// # Arguments
/// 
/// * `pos1` - The first tile position
/// * `pos2` - The second tile position
/// 
/// # Returns
/// 
/// The distance in tiles as a `f32`
/// 
/// # Examples
/// 
/// ```rust
/// let distance = tile_distance(
///     TilePos::new(0, 0),
///     TilePos::new(3, 4)
/// );
/// assert_eq!(distance, 5.0);
/// ```
pub fn tile_distance(pos1: TilePos, pos2: TilePos) -> f32 {
    // Implementation
}
```

#### Module Documentation
```rust
//! Network handling for cross-platform WebSocket communication.
//! 
//! This module provides abstractions over native WebSocket libraries
//! and browser WebSocket APIs, allowing the same client code to work
//! in both native and WASM environments.
//! 
//! # Platform Support
//! 
//! - **Native**: Uses `ws` crate for WebSocket connections
//! - **WASM**: Uses `web-sys` browser WebSocket API
//! 
//! # Examples
//! 
//! ```rust
//! let mut client = NetworkClient::new("ws://localhost:8080")?;
//! client.send_message(ClientMessage::Join("Alice".to_string()))?;
//! ```
```

### Testing Conventions

#### Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_tile_distance_basic() {
        // Arrange
        let pos1 = TilePos::new(0, 0);
        let pos2 = TilePos::new(3, 4);
        
        // Act
        let distance = tile_distance(pos1, pos2);
        
        // Assert
        assert_eq!(distance, 5.0);
    }

    #[test]
    fn test_player_movement_bounds() {
        // Test with descriptive names and clear arrange/act/assert
        let mut game_state = create_test_game_state();
        let player_id = add_test_player(&mut game_state);
        
        let result = move_player(&mut game_state, player_id, Direction::North);
        
        assert!(result.is_ok());
        assert_eq!(get_player_position(&game_state, player_id).y, 1);
    }
}
```

#### Test Naming
- **Function names:** `test_{feature}_{scenario}` (e.g., `test_movement_out_of_bounds`)
- **Descriptive assertions:** Clear failure messages
- **Test data:** Use helper functions for consistent test setup

### Performance Guidelines

#### Hot Path Optimization
```rust
// ✅ Use inline hints for performance-critical functions
#[inline]
pub fn tile_distance(pos1: TilePos, pos2: TilePos) -> f32 {
    let dx = (pos2.x - pos1.x) as f32;
    let dy = (pos2.y - pos1.y) as f32;
    (dx * dx + dy * dy).sqrt()
}

// ✅ Cache expensive calculations
struct CachedVisibility {
    camera_pos: WorldPos,
    visible_tiles: HashSet<TilePos>,
}

impl CachedVisibility {
    fn update_if_needed(&mut self, current_camera: &Camera) {
        if self.camera_pos != current_camera.position {
            self.recalculate_visibility(current_camera);
        }
    }
}
```

#### Memory Management
```rust
// ✅ Use appropriate collection types
use std::collections::{HashMap, HashSet, VecDeque};

// HashMap for O(1) lookups by ID
type EntityStorage = HashMap<EntityId, Entity>;

// HashSet for membership testing
type VisibleTiles = HashSet<TilePos>;

// VecDeque for FIFO message queues
type MessageQueue = VecDeque<NetworkMessage>;
```

### Platform-Specific Code

#### Conditional Compilation
```rust
// ✅ Clear platform separation
#[cfg(target_arch = "wasm32")]
mod web_network {
    use web_sys::WebSocket;
    // Web-specific implementation
}

#[cfg(not(target_arch = "wasm32"))]
mod native_network {
    use ws::WebSocket;
    // Native-specific implementation
}

// ✅ Unified interface
pub use self::network_impl::NetworkClient;

#[cfg(target_arch = "wasm32")]
mod network_impl {
    pub use super::web_network::NetworkClient;
}

#[cfg(not(target_arch = "wasm32"))]
mod network_impl {
    pub use super::native_network::NetworkClient;
}
```

### Project-Specific Patterns

#### Rendering Abstractions
```rust
// ✅ Use rendering primitives for consistency
use crate::rendering::primitives::{ArrowRenderer, TileHighlight};

// Draw arrows using abstraction
ArrowRenderer::draw_arrow_centered_in_tile(
    tile_pos,
    Direction::Up,
    ArrowStyle::default().with_color(BLUE)
);

// Highlight tiles consistently
TileHighlight::draw_tile(
    selected_tile, 
    camera_offset, 
    TileHighlightStyle::selection()
);
```

#### Math Utilities
```rust
// ✅ Use tile math utilities instead of manual calculations
use shared::tile_math::{TileDistance, TileNavigation};

// Distance calculations
let distance = TileDistance::tile_distance(pos1, pos2);
let in_range = TileDistance::within_tile_radius(center, target, 3.0);

// Navigation helpers
let adjacent = TileNavigation::adjacent_tiles(current, Some(bounds));
let path_tiles = TileNavigation::line_of_tiles(start, end);
```

### Git & Development Workflow

#### Commit Messages
```
Type: Brief description in present tense

Longer explanation of what changed and why, if needed.

Examples:
- "feat: Add hybrid tile-entity system for world representation"
- "fix: Resolve camera offset calculation in grass background"
- "refactor: Extract coordinate conversion utilities to shared module"
- "docs: Update development guide with new build instructions"
```

#### Branch Naming
- **Features:** `feature/hybrid-tile-system`
- **Bug fixes:** `fix/camera-offset-calculation`
- **Documentation:** `docs/update-development-guide`
- **Refactoring:** `refactor/coordinate-abstractions`

### Code Review Guidelines

#### Review Checklist
- ✅ **Follows naming conventions** - snake_case, PascalCase appropriately used
- ✅ **Error handling** - Proper use of Result types and error propagation
- ✅ **Documentation** - Public APIs documented with examples
- ✅ **Testing** - New functionality has corresponding tests
- ✅ **Performance** - No obvious performance regressions
- ✅ **Platform compatibility** - Works on both native and WASM
- ✅ **Abstractions usage** - Uses coordinate and rendering abstractions

#### Anti-patterns to Flag
- ❌ Manual tile math calculations
- ❌ Direct use of TILE_SIZE constants
- ❌ Platform-specific code without proper conditional compilation
- ❌ Missing error handling
- ❌ Hardcoded magic numbers
- ❌ Inconsistent naming patterns

These conventions ensure consistency across the codebase and align with the project's architectural decisions and performance requirements.
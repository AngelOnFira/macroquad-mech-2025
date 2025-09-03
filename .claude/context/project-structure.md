---
created: 2025-09-03T18:04:12Z
last_updated: 2025-09-03T18:04:12Z
version: 1.0
author: Claude Code PM System
---

# Project Structure

## Workspace Organization

**Type:** Rust Cargo Workspace  
**Crates:** 5 member crates with shared dependencies  
**Resolver:** Version 2

```
mech-battle-arena/
├── Cargo.toml              # Workspace configuration
├── Cargo.lock              # Dependency lock file
├── justfile                # Task runner commands
├── devtabs.yaml            # Development environment config
└── build_js.sh             # WASM build script
```

## Core Crates Structure

### Client (`client/`)
- **Purpose:** Game client (native + WASM)
- **Technology:** Macroquad for cross-platform graphics
- **Key Files:**
  - `src/main.rs` - Client entry point
  - `src/network_web.rs` - WebSocket for browsers
  - `src/bin/demo.rs` - Interactive hybrid system demo

### Server (`server/`)
- **Purpose:** Authoritative multiplayer game server
- **Technology:** Axum WebSocket server with tokio
- **Key Files:**
  - `src/game.rs` - Core game logic
  - `src/entity_storage.rs` - Entity management system
  - `src/client.rs` - Client connection handling

### Shared (`shared/`)
- **Purpose:** Common types and protocol definitions
- **Key Files:**
  - `src/messages.rs` - Network protocol definitions
  - `src/types.rs` - Game type definitions
  - `src/tile_entity.rs` - Hybrid tile system core
  - `src/vision.rs` - Raycasting vision system
  - `src/components.rs` - ECS components for entities

### AI (`ai/`)
- **Purpose:** AI player system and decision making
- **Key Files:**
  - `src/lib.rs` - Main AI interface
  - `src/decision.rs` - Decision making logic
  - `src/perception.rs` - AI perception systems
  - `src/personality.rs` - AI personality traits
  - `src/communication.rs` - AI communication
  - `src/utility.rs` - AI utility functions

### Debug Client (`debug-client/`)
- **Purpose:** Debugging and testing tools
- **Technology:** Specialized client for development

## Configuration Files

### Development Environment
- **`devtabs.yaml`** - Multi-process development orchestration
- **`justfile`** - Build automation and common tasks
- **`.vscode/`** - VS Code configuration
- **`.devcontainer/`** - Development container setup

### Build Configuration
- **`Cargo.toml`** - Workspace and dependency management
- **`build_js.sh`** - WASM build automation
- **Profile configurations:**
  - `debug-opt` - Optimized debug builds
  - WASM-specific optimizations

## Documentation Structure

### Project Documentation
- **`CLAUDE.md`** - Development guide and conventions
- **`docs/`** - Design documents and architecture guides
  - `HYBRID_TILE_ENTITY_SYSTEM.md` - Tile system documentation
  - `MECH_INTERIOR_DESIGN.md` - Mech interior implementation

### Development Documentation  
- **`AGENTS.md`** - AI agent documentation
- **`COMMANDS.md`** - Command reference
- **`AI_DEBUG_CLIENT.md`** - AI debugging guide
- **`AI_IMPLEMENTATION.md`** - AI implementation details
- **`WASM_BUILD_FIXES.md`** - WASM build troubleshooting

## Build Artifacts

### Target Directory (`target/`)
- **Size:** ~69MB of build artifacts
- **Profiles:** debug, release, debug-opt
- **Targets:** native, wasm32-unknown-unknown

### Distribution (`dist/`)
- WASM build outputs
- Web assets and binaries

## Log Management

### Log Files
- **`logs/`** - Runtime logs directory
- **`server.log`** - Current server logs
- **`web_server.log`** - Web server logs
- Historical log files from development sessions

## File Naming Conventions

### Rust Files
- **Module files:** snake_case (e.g., `tile_entity.rs`)
- **Binary files:** snake_case in `src/bin/`
- **Test files:** `mod tests` or `tests/` directory

### Documentation
- **All caps:** Project-level docs (e.g., `CLAUDE.md`)
- **Lowercase:** Technical docs in `docs/`
- **Snake case:** Generated files

## Directory Access Patterns

### Most Active Directories
1. **`client/src/`** - Client development
2. **`server/src/`** - Server logic
3. **`shared/src/`** - Protocol and types
4. **`docs/`** - Architecture documentation
5. **`.claude/`** - Development automation

### Build Dependencies
- **Root:** Workspace configuration
- **Each crate:** Individual Cargo.toml files
- **Shared deps:** Defined at workspace level

## External Dependencies

### Asset Management
- No external asset directories currently
- Web assets served from `dist/`
- All resources embedded or generated

### Integration Points
- **Git:** Standard git repository structure
- **GitHub:** Remote repository integration
- **DevTabs:** Process orchestration
- **Just:** Task automation
---
created: 2025-09-03T18:04:12Z
last_updated: 2025-09-03T18:04:12Z
version: 1.0
author: Claude Code PM System
---

# Technical Context

## Core Technology Stack

### Programming Language
- **Rust:** Primary language across all crates
- **Edition:** 2021 (modern Rust features)
- **Toolchain:** Stable Rust with WASM support

### Frameworks & Libraries

#### Game Engine & Graphics
- **Macroquad:** Cross-platform game framework
  - Native desktop support (OpenGL)
  - WASM/WebGL support for browsers
  - 2D rendering with hardware acceleration

#### Networking
- **Axum:** Modern async web framework for server
- **Tokio:** Async runtime for server-side operations
- **WebSockets:** Real-time multiplayer communication
  - Native: `ws` crate
  - WASM: `web-sys` WebSocket API

#### Serialization & Data
- **Serde:** Serialization framework with derive macros
- **serde_json:** JSON serialization for network protocol
- **UUID:** Entity identification with serde support

## Workspace Dependencies

### Shared Dependencies
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"
uuid = { version = "1.10", features = ["serde"] }
thiserror = "1.0"
anyhow = "1.0"
```

### Development & Debugging
- **Tracing:** Structured logging and profiling
  - `tracing = "0.1"`
  - `tracing-subscriber = "0.3"`
  - `tracing-tree = "0.4"`
- **Profiling:** Performance analysis tools
  - `profiling = "1.0"`

## Build System & Tooling

### Task Automation
- **Just:** Modern command runner (replaces Make)
  - Build automation
  - Development workflows
  - Multi-target builds
- **DevTabs:** Multi-process development orchestration
  - Automated server startup
  - WASM rebuild on changes
  - Log aggregation

### Build Profiles

#### Debug-Optimized Profile
```toml
[profile.debug-opt]
inherits = "dev"
opt-level = 2              # Performance optimization
debug = true               # Keep debug symbols
lto = "thin"              # Link-time optimization
codegen-units = 1         # Better optimization
panic = "abort"           # Smaller binaries
```

## Platform Support

### Native Targets
- **Linux:** Primary development platform
- **Windows:** Cross-compilation supported
- **macOS:** Cross-compilation supported

### Web Assembly (WASM)
- **Target:** `wasm32-unknown-unknown`
- **Build Tool:** `wasm-pack` (manual builds)
- **Integration:** Direct WebAssembly loading (not wasm-bindgen)
- **Constraint:** Cannot use wasm-bindgen dependencies (Macroquad conflict)

## Development Environment

### Required Tools
- **Rust Toolchain:** Latest stable
- **WASM Target:** `rustup target add wasm32-unknown-unknown`
- **Just:** Task runner installation
- **Node.js:** For web server and Playwright tests

### Optional Tools
- **DevTabs:** Process orchestration
- **VS Code:** IDE configuration provided
- **Docker:** Container development support

## Dependency Management

### Version Strategy
- **Workspace-level:** Shared dependencies defined once
- **Crate-specific:** Additional features per crate
- **Lock file:** Cargo.lock committed for reproducible builds

### Known Constraints
- **WASM Compatibility:** No `wasm-bindgen` dependencies
- **Macroquad Conflicts:** Avoid web-sys dependent crates
- **Performance:** Profiling dependencies for optimization

## Network Architecture

### Protocol Design
- **Format:** JSON over WebSockets
- **Messages:** Strongly-typed enums with serde
- **Client-Server:** Authoritative server model
- **Real-time:** Low-latency game state updates

### Connection Handling
- **Native Clients:** Direct WebSocket connections
- **Web Clients:** Browser WebSocket API
- **Multiplayer:** Multiple concurrent connections
- **State Sync:** Centralized game state management

## Performance Considerations

### Optimization Features
- **LTO:** Link-time optimization enabled
- **Profiling Integration:** Built-in performance monitoring
- **WASM Optimization:** Size and speed optimizations
- **Debug Symbols:** Preserved for profiling

### Resource Management
- **Memory:** Rust ownership model for safety
- **Network:** Efficient JSON serialization
- **Graphics:** Hardware-accelerated rendering
- **Concurrency:** Async/await for I/O operations

## Testing Infrastructure

### Test Types
- **Unit Tests:** Rust built-in test framework
- **Integration:** Cross-crate functionality
- **Browser Tests:** Playwright for WASM validation
- **Multiplayer:** Multi-client testing support

### Quality Assurance
- **Linting:** Clippy for code quality
- **Formatting:** rustfmt for consistency
- **Type Safety:** Strong static typing
- **Memory Safety:** Rust guarantees

## External Integrations

### Version Control
- **Git:** Standard git workflow
- **GitHub:** Remote repository hosting
- **Branches:** Main branch development

### Build & Deployment
- **Local Development:** Just + DevTabs
- **Cross-compilation:** Multiple target support
- **Web Deployment:** Static WASM + assets
- **Native Distribution:** Cargo build system
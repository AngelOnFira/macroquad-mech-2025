# Mech Battle Arena - Development Guide

This guide helps Claude Code understand and work with the Mech Battle Arena project.

## Project Overview

A multiplayer web game where teams control giant mechs from the inside. Players can:
- Move around the world and inside mechs
- Operate various stations (weapons, shields, engines, etc.)
- Collect resources to upgrade their mechs
- Engage in team vs team combat

## Quick Start

```bash
# Start everything for development
just dev

# Or step by step:
just build-web      # Build WebAssembly client
just server         # Start game server (in one terminal)
just web-server     # Start web server (in another terminal)
```

Then open http://localhost:8080 in multiple browser tabs to test multiplayer.

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

## Architecture

### Workspace Structure
- `server/` - Axum WebSocket server, authoritative game state
- `client/` - Macroquad game client (native and WASM)
- `shared/` - Protocol definitions and shared types

### Key Technologies
- **Networking**: WebSockets (native: `ws`, web: `web-sys`)
- **Graphics**: Macroquad (works in both native and WASM)
- **Serialization**: JSON via serde_json
- **Server**: Axum with tokio async runtime

### Network Protocol
All messages are JSON-serialized enums defined in `shared/src/messages.rs`:
- `ClientMessage`: Player inputs, join requests, station controls
- `ServerMessage`: State updates, events, confirmations

## Development Workflow

1. **Make Changes**: Edit code in appropriate crate
2. **Test Locally**: 
   ```bash
   just dev  # Starts server + web server
   # Open http://localhost:8080 in two tabs
   ```
3. **Check Quality**: 
   ```bash
   just check  # Lints and type checks
   just test   # Run tests
   ```

## Important Files

- `justfile` - All development commands
- `shared/src/messages.rs` - Network protocol
- `shared/src/types.rs` - Game types
- `server/src/game.rs` - Core game logic
- `client/src/main.rs` - Client entry point
- `client/src/network_web.rs` - WebSocket for browsers

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

- **Movement**: WASD keys, grid-based
- **Mechs**: 3 floors, multiple stations, team-owned
- **Resources**: 4 types, used for upgrades/repairs
- **Combat**: Laser (instant) and projectile weapons
- **Upgrades**: Improve weapons, shields, engines

## WebAssembly Specifics

The client uses conditional compilation:
- `#[cfg(target_arch = "wasm32")]` - Web-specific code
- `#[cfg(not(target_arch = "wasm32"))]` - Native-specific code

Key differences:
- Web uses `web-sys` WebSocket, native uses `ws` crate
- Web connects to page origin, native to localhost
- Web uses console logging, native uses env_logger
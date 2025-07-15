# Migration to Trunk - WebAssembly Build Fix

This document summarizes the changes made to fix WebAssembly instantiation errors by migrating from manual WASM builds to Trunk.

## Problem
The project was experiencing WebAssembly instantiation errors:
```
TypeError: WebAssembly.instantiate(): Import #12 "__wbindgen_placeholder__": module is not an object or function
```

This was caused by dependency conflicts between wasm-bindgen (pulled in by uuid crate) and Macroquad's web loader.

## Solution
Migrated to Trunk for automatic WASM compilation and serving.

## Changes Made

### 1. Workspace Dependencies Cleanup
**File: `/Cargo.toml`**
- Removed server-specific dependencies from workspace (tokio, axum, tower, env_logger, macroquad)
- Kept only truly shared dependencies: serde, serde_json, log, uuid, thiserror, anyhow

### 2. Server Dependencies
**File: `/server/Cargo.toml`**
- Added server-specific dependencies directly:
  - tokio = { version = "1.40", features = ["full"] }
  - axum = { version = "0.7", features = ["ws"] }
  - tower = "0.5"
  - env_logger = "0.11"

### 3. Client Dependencies
**File: `/client/Cargo.toml`**
- Fixed uuid dependency for WASM to avoid wasm-bindgen conflicts
- Added web-sys for WebSocket support with Trunk
- Removed problematic "v4" and "js" features from uuid

### 4. Trunk Setup
**New Files:**
- `/client/index.html` - HTML template for Trunk
- `/client/Trunk.toml` - Trunk configuration with WebSocket proxy

### 5. Updated Build System
**File: `/justfile`**
- Replaced manual WASM build commands with Trunk
- Updated `dev` command to use `trunk serve`
- Removed old web-server commands
- Added `install-trunk` command

**File: `/devtabs.yaml`**
- Replaced "web-server" and "build-wasm" with "trunk-serve"

### 6. WebSocket Implementation
**File: `/client/src/main.rs`**
- Switched from `network_web_macroquad` to `network_web`
- Now uses web-sys WebSocket with proper wasm-bindgen support

### 7. Other Crate Fixes
- Updated `/shared/Cargo.toml` to use macroquad directly
- Updated `/ai/Cargo.toml` to use workspace dependencies
- Updated `/debug-client/Cargo.toml` to use env_logger directly

## Files to Remove
Run `./cleanup_old_web.sh` to remove:
- `/web_build/` directory
- `/build_web.sh`
- `/build_web_macroquad.sh`
- `/client/src/network_web_macroquad.rs`

## How to Use

1. **Install Trunk**:
   ```bash
   cargo install trunk
   ```

2. **Clean and rebuild**:
   ```bash
   just clean-all
   just build
   ```

3. **Run development environment**:
   ```bash
   just dev
   ```
   Or use DevTabs which now uses Trunk.

4. **Build for release**:
   ```bash
   just release-web
   ```

## Benefits
- Automatic WASM bundling with proper wasm-bindgen glue code
- Hot reload during development
- Cleaner dependency separation
- No more WebAssembly instantiation errors
- Simpler build process
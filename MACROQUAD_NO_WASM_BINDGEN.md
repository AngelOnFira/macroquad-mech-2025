# Macroquad Without wasm-bindgen

## Summary

We've successfully configured the project to work with macroquad's native WASM loader, completely avoiding wasm-bindgen and its compatibility issues.

## What Changed

1. **Removed ALL wasm-bindgen dependencies**:
   - Removed `wasm-bindgen`, `web-sys`, and `js-sys` from client/Cargo.toml
   - Created a stub network implementation for WASM builds

2. **Simple build process**:
   - `build-simple.sh` - Just runs cargo build and copies files
   - No patching required
   - No complex build tools

3. **Pure macroquad HTML**:
   - Uses macroquad's JS bundle
   - Loads WASM directly with `load("./client.wasm")`
   - No module imports or wasm-bindgen glue code

## Trade-offs

- ✅ **Pros**:
  - No more "__wbindgen_placeholder__" errors
  - Simple, reliable build process
  - Smaller WASM size (no wasm-bindgen overhead)
  - Works exactly as macroquad intended

- ❌ **Cons**:
  - No WebSocket support in WASM (yet)
  - Can't use web-sys APIs
  - Limited to macroquad's built-in features

## WebSocket Support

Currently, the WASM build runs in offline mode. To add WebSocket support, options include:

1. Use macroquad's networking (if it supports WebSocket)
2. Implement JavaScript FFI through miniquad
3. Create a custom solution using macroquad's JS interop

## Usage

```bash
# Build WASM
just build-web

# Start dev environment
just dev

# Release build
just release-web
```

The game will now load in the browser without any wasm-bindgen errors!
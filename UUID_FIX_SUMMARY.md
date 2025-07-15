# UUID Dependency Fix for WASM Builds

## Problem
The shared crate was using `Uuid::new_v4()` which requires the `v4` feature of the uuid crate. This feature pulls in dependencies that conflict with WASM builds, causing WebAssembly instantiation errors.

## Solution
1. **Conditional Dependencies**: Configure uuid differently for native vs WASM targets in `shared/Cargo.toml`:
   - Native builds: `uuid = { version = "1.10", features = ["serde", "v4"] }`
   - WASM builds: `uuid = { version = "1.10", features = ["serde"] }`

2. **UUID Generation Helper**: Created `shared/src/uuid_gen.rs` that provides:
   - `new_uuid()` function that works on native targets (calls `Uuid::new_v4()`)
   - Panics on WASM targets (client should never generate UUIDs)

3. **Code Updates**: Replaced all `Uuid::new_v4()` calls with `new_uuid()` in:
   - `shared/src/stations.rs`
   - `shared/src/mech_layout.rs`
   - `shared/src/spatial.rs`

4. **Conditional Compilation**: Made `object_pool` module only available for non-WASM targets since it's server-only.

## Result
- Server can generate UUIDs as needed
- Client can deserialize UUIDs received from server
- No wasm-bindgen conflicts in WASM builds
- Clean separation of server-side and client-side functionality

## Testing
```bash
# Test native build
cargo build --bin server
cargo build --bin client

# Test WASM build
cd client && cargo build --target wasm32-unknown-unknown --no-default-features

# Or run the full test script
chmod +x test_build.sh
./test_build.sh
```
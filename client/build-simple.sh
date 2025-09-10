#!/bin/bash
# Simple build script for macroquad WASM without wasm-bindgen

set -e

PROFILE="debug-opt"
CARGO_FLAGS="--profile debug-opt"
FEATURES="web,profiling-wasm"
if [[ "$1" == "--release" ]]; then
    PROFILE="release"
    CARGO_FLAGS="--release"
    FEATURES="web"
elif [[ "$1" == "--pages" ]]; then
    PROFILE="release-size"
    CARGO_FLAGS="--profile release-size"
    FEATURES="web"
fi

echo "Building WASM for profile: $PROFILE"

# Build the WASM with web and profiling features
cargo build --target wasm32-unknown-unknown --no-default-features --features $FEATURES $CARGO_FLAGS
# cargo build --target wasm32-unknown-unknown --no-default-features --features web,profiling $CARGO_FLAGS

# Create dist directory
mkdir -p ../dist

# Copy the WASM file
cp ../target/wasm32-unknown-unknown/$PROFILE/client.wasm ../dist/

# Copy the HTML file
cp index-macroquad.html ../dist/index.html

# Copy the network bindings JavaScript
cp network_bindings.js ../dist/

# Run ../build_js.sh
$(cd .. && ./build_js.sh)

echo "Build complete! Files in ../dist/"
echo "Start a web server in the dist directory to run the game."
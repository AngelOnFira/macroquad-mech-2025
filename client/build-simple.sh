#!/bin/bash
# Simple build script for macroquad WASM without wasm-bindgen

set -e

PROFILE="debug"
CARGO_FLAGS=""
if [[ "$1" == "--release" ]]; then
    PROFILE="release"
    CARGO_FLAGS="--release"
fi

echo "Building WASM for profile: $PROFILE"

# Build the WASM
cargo build --target wasm32-unknown-unknown --no-default-features $CARGO_FLAGS

# Create dist directory
mkdir -p ../dist

# Copy the WASM file
cp ../target/wasm32-unknown-unknown/$PROFILE/client.wasm ../dist/

# Copy the HTML file
cp index-macroquad.html ../dist/index.html

# Copy the network bindings JavaScript
cp network_bindings.js ../dist/

echo "Build complete! Files in ../dist/"
echo "Start a web server in the dist directory to run the game."
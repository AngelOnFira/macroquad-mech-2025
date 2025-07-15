#!/bin/bash

echo "Testing WASM release build..."
echo "============================"

cd client

# First check if we have the WASM target
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "ERROR: WASM target not installed"
    echo "Run: rustup target add wasm32-unknown-unknown"
    exit 1
fi

# Try the release build
echo -e "\nBuilding client for WASM (release mode)..."
if cargo build --target wasm32-unknown-unknown --release --no-default-features 2>&1; then
    echo "✓ WASM release build successful"
else
    echo "✗ WASM release build failed"
    echo -e "\nTrying with verbose output to see the error:"
    cargo build --target wasm32-unknown-unknown --release --no-default-features -v
fi
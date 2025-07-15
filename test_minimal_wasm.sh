#!/bin/bash

echo "Minimal WASM Build Test"
echo "======================="
echo

# Test 1: Can we build shared for WASM?
echo "1. Testing shared crate for WASM..."
cd shared
if cargo build --target wasm32-unknown-unknown 2>&1; then
    echo "✓ Shared builds for WASM"
else
    echo "✗ Shared failed for WASM"
    echo "Retrying with verbose output:"
    cargo build --target wasm32-unknown-unknown -v
fi
cd ..
echo

# Test 2: Can we build client with minimal features?
echo "2. Testing client crate for WASM (minimal)..."
cd client
if cargo build --target wasm32-unknown-unknown --no-default-features 2>&1; then
    echo "✓ Client builds for WASM"
else
    echo "✗ Client failed for WASM"
    echo "Retrying with verbose output:"
    cargo build --target wasm32-unknown-unknown --no-default-features -v
fi
cd ..
echo

# Test 3: Check specific dependencies
echo "3. Checking dependency tree for WASM..."
cd client
cargo tree --target wasm32-unknown-unknown --no-default-features 2>&1 | head -20
cd ..
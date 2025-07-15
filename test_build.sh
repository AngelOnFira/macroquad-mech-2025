#!/bin/bash

echo "Testing build configuration..."
echo "=============================="
echo "This test verifies the uuid dependency fix for WASM builds"
echo

# Check server build
echo -e "\n1. Building server..."
if cargo build --bin server 2>&1; then
    echo "✓ Server build successful"
else
    echo "✗ Server build failed"
fi

# Check client build
echo -e "\n2. Building client..."
if cargo build --bin client 2>&1; then
    echo "✓ Client build successful"
else
    echo "✗ Client build failed"
fi

# Check WASM build
echo -e "\n3. Checking WASM target..."
if rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "✓ WASM target installed"
    
    echo -e "\n4. Building client for WASM..."
    if cd client && cargo build --target wasm32-unknown-unknown --no-default-features 2>&1; then
        echo "✓ WASM build successful"
    else
        echo "✗ WASM build failed"
    fi
    cd ..
else
    echo "✗ WASM target not installed"
    echo "  Run: rustup target add wasm32-unknown-unknown"
fi

# Check workspace
echo -e "\n5. Checking entire workspace..."
if cargo check --workspace 2>&1; then
    echo "✓ Workspace check successful"
else
    echo "✗ Workspace check failed"
fi

echo -e "\n=============================="
echo "Build test complete!"
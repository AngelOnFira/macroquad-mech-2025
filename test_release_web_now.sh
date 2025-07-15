#!/bin/bash

echo "Testing 'just release-web' after fixes..."
echo "========================================"
echo

# Run the command
just release-web

if [ $? -eq 0 ]; then
    echo
    echo "✅ SUCCESS! WASM release build completed!"
    echo
    echo "Build artifacts:"
    ls -la dist/ 2>/dev/null || echo "No dist/ directory found (Trunk may have failed)"
    ls -la ../dist/ 2>/dev/null || echo "Checking parent directory..."
    
    # Check WASM file size
    if [ -f ../dist/*.wasm ]; then
        echo
        echo "WASM file size:"
        ls -lh ../dist/*.wasm
    fi
else
    echo
    echo "❌ Build failed. See errors above."
fi
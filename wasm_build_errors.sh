#!/bin/bash

echo "Capturing WASM build errors..."
echo "=============================="
echo

cd client

echo "1. Attempting WASM release build..."
echo "Command: cargo build --target wasm32-unknown-unknown --release --no-default-features"
echo

# Run the build and capture all output
cargo build --target wasm32-unknown-unknown --release --no-default-features 2>&1 | tee ../wasm_build_output.txt

# Check if it failed
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo
    echo "Build failed! Extracting error information..."
    echo
    
    # Extract just the errors
    echo "=== ERRORS ==="
    grep -E "error\[E[0-9]+\]|error:" ../wasm_build_output.txt
    
    echo
    echo "=== WARNINGS ==="
    grep -E "warning:" ../wasm_build_output.txt | head -10
    
    echo
    echo "Full output saved to: wasm_build_output.txt"
else
    echo
    echo "Build succeeded!"
fi

cd ..
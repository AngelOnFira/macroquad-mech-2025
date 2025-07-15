#!/bin/bash

echo "Running 'just release-web' to capture errors..."
echo "================================================"
echo

# First check if just is available
if ! command -v just &> /dev/null; then
    echo "ERROR: 'just' command not found"
    echo "Please install just: cargo install just"
    exit 1
fi

# Run the command and capture output
echo "Executing: just release-web"
echo
just release-web 2>&1 | tee release_web_output.log

echo
echo "================================================"
echo "Output saved to release_web_output.log"

# Also try running the underlying command directly
echo
echo "Trying direct cargo build..."
cd client && cargo build --target wasm32-unknown-unknown --release --no-default-features 2>&1 | tee ../direct_cargo_output.log

echo
echo "Direct cargo output saved to direct_cargo_output.log"
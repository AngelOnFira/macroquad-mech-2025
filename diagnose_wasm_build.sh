#!/bin/bash

echo "WASM Build Diagnostics"
echo "====================="
echo

# Function to run a command and capture both stdout and stderr
run_command() {
    local cmd="$1"
    local desc="$2"
    
    echo "Running: $cmd"
    echo "Description: $desc"
    echo "---"
    
    # Run the command and capture output
    output=$($cmd 2>&1)
    exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        echo "✓ Success"
    else
        echo "✗ Failed with exit code: $exit_code"
        echo "Error output:"
        echo "$output" | grep -E "error\[E[0-9]+\]|error:|failed" | head -20
    fi
    echo
    
    return $exit_code
}

# Check environment
echo "=== Environment Check ==="
run_command "rustc --version" "Rust compiler version"
run_command "cargo --version" "Cargo version"
run_command "rustup target list | grep wasm32-unknown-unknown" "WASM target status"

# Try to build step by step
echo "=== Build Tests ==="

echo "Step 1: Building shared crate for WASM"
cd shared
run_command "cargo build --target wasm32-unknown-unknown" "Build shared for WASM"
cd ..

echo "Step 2: Checking client dependencies"
cd client
run_command "cargo tree --target wasm32-unknown-unknown --no-default-features | grep -E '^[^├└]' | head -10" "Top-level dependencies"

echo "Step 3: Building client for WASM (check only)"
run_command "cargo check --target wasm32-unknown-unknown --no-default-features" "Check client for WASM"

echo "Step 4: Building client for WASM (actual build)"
run_command "cargo build --target wasm32-unknown-unknown --no-default-features" "Build client for WASM"

echo "Step 5: Release build"
run_command "cargo build --target wasm32-unknown-unknown --release --no-default-features" "Release build for WASM"
cd ..

echo "=== Checking for common issues ==="

# Check for println! usage
echo "Checking for println! usage (not supported in WASM):"
grep -n "println!" client/src/*.rs shared/src/*.rs 2>/dev/null | head -5 || echo "✓ No println! found"
echo

# Check for std::thread usage outside of cfg
echo "Checking for thread usage outside of cfg blocks:"
grep -n "std::thread" client/src/*.rs | grep -v "cfg" | head -5 || echo "✓ No unconditioned thread usage found"
echo

echo "Diagnostics complete!"
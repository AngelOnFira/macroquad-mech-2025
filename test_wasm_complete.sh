#!/bin/bash

echo "Complete WASM Build Test"
echo "========================"
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if WASM target is installed
echo "1. Checking WASM target..."
if rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo -e "${GREEN}✓${NC} WASM target installed"
else
    echo -e "${RED}✗${NC} WASM target not installed"
    echo "Installing WASM target..."
    rustup target add wasm32-unknown-unknown
fi
echo

# Check shared crate for WASM
echo "2. Checking shared crate (WASM)..."
if cd shared && cargo check --target wasm32-unknown-unknown 2>&1; then
    echo -e "${GREEN}✓${NC} Shared crate builds for WASM"
else
    echo -e "${RED}✗${NC} Shared crate failed WASM build"
    cd shared && cargo check --target wasm32-unknown-unknown
fi
cd ..
echo

# Check client crate for WASM (debug)
echo "3. Checking client crate (WASM debug)..."
if cd client && cargo check --target wasm32-unknown-unknown --no-default-features 2>&1; then
    echo -e "${GREEN}✓${NC} Client crate builds for WASM (debug)"
else
    echo -e "${RED}✗${NC} Client crate failed WASM build (debug)"
    cd client && cargo check --target wasm32-unknown-unknown --no-default-features
fi
cd ..
echo

# Check client crate for WASM (release)
echo "4. Building client crate (WASM release)..."
if cd client && cargo build --target wasm32-unknown-unknown --release --no-default-features 2>&1; then
    echo -e "${GREEN}✓${NC} Client crate builds for WASM (release)"
    WASM_SIZE=$(ls -lh ../target/wasm32-unknown-unknown/release/client.wasm | awk '{print $5}')
    echo "   WASM size: $WASM_SIZE"
else
    echo -e "${RED}✗${NC} Client crate failed WASM build (release)"
fi
cd ..
echo

# Check if Trunk is installed
echo "5. Checking Trunk..."
if command -v trunk &> /dev/null; then
    echo -e "${GREEN}✓${NC} Trunk is installed"
    trunk --version
else
    echo -e "${RED}✗${NC} Trunk not installed"
    echo "   Install with: cargo install trunk"
fi
echo

echo "========================"
echo "Test complete!"
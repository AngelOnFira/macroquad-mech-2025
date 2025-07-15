#!/bin/bash

echo "Cleaning up old web build files..."

# Remove old build scripts
rm -f build_web.sh
rm -f build_web_macroquad.sh

# Remove old web build directory
rm -rf web_build/

# Remove the macroquad network implementation
rm -f client/src/network_web_macroquad.rs

echo "Cleanup complete!"
echo
echo "Next steps:"
echo "1. Install Trunk: cargo install trunk"
echo "2. Run the dev environment: just dev"
echo "3. Or use DevTabs for development"
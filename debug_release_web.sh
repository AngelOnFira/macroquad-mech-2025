#!/bin/bash

echo "Debugging 'just release-web' command"
echo "===================================="
echo

# Check if just is available
if ! command -v just &> /dev/null; then
    echo "ERROR: 'just' is not installed"
    echo "Install with: cargo install just"
    exit 1
fi

# Check current directory
echo "Current directory: $(pwd)"
echo

# Check if we're in the right place
if [ ! -f "justfile" ]; then
    echo "ERROR: justfile not found in current directory"
    exit 1
fi

# Run just release-web with error capture
echo "Running: just release-web"
echo "------------------------"

# Create a temporary script to capture the exact error
cat > temp_release_web.sh << 'EOF'
#!/bin/bash
set -e  # Exit on error
set -x  # Print commands

# This is what the justfile does
echo "Building WASM release..."

# First ensure WASM target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "Installing WASM target..."
    rustup target add wasm32-unknown-unknown
fi

# Build with Trunk if available, otherwise use cargo
if command -v trunk &> /dev/null; then
    echo "Using Trunk..."
    cd client && trunk build --release
else
    echo "Trunk not found, using cargo build..."
    cd client && cargo build --target wasm32-unknown-unknown --release --no-default-features
    echo "Note: Install Trunk for automatic bundling: cargo install trunk"
fi
EOF

chmod +x temp_release_web.sh

# Run it
./temp_release_web.sh 2>&1 | tee release_web_debug.log

# Clean up
rm temp_release_web.sh

echo
echo "------------------------"
echo "Output saved to: release_web_debug.log"

# If it failed, show the key error
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo
    echo "Build failed! Key errors:"
    grep -E "error\[E[0-9]+\]|error:|failed to" release_web_debug.log | head -10
fi
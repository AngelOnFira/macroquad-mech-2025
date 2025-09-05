# Mech Battle Arena - Task Runner

# Default task - show available commands
default:
    @just --list

# Build all components
build: build-server build-client
    @echo "Note: Run 'just release-web' to build WASM client"

# Build the server
build-server:
    cargo build --bin server

# Build the native client
build-client:
    cargo build --bin client

# Build WebAssembly client
build-web:
    cd client && ./build-simple.sh

# Run the server
server:
    RUST_LOG=info cargo run --bin server

# Run the native client
client:
    RUST_LOG=info cargo run --bin client

# Run test client with custom name
test-client name="TestPlayer":
    cargo run --bin test_client {{name}}

# Start web server with live reload using browser-sync
web-serve:
    #!/bin/bash
    just build-web
    npx browser-sync start --server dist --files "dist/**/*" --no-notify --port 8080

# Start web server only (no build) - used by VS Code tasks
web-serve-only:
    npx browser-sync start --server dist --files "dist/**/*" --no-notify --port 8080

# Watch and auto-rebuild WASM client using bacon
watch-web:
    bacon build-web

# Watch and auto-rebuild server using bacon
watch-server:
    bacon build-server

# Watch and auto-rebuild everything using bacon
watch-all:
    bacon build-all

# Watch and run server with auto-restart using bacon
watch-run-server:
    bacon run-server


# Build and run the hybrid tile demo
build-demo:
    cd client && ./build-demo.sh

# Run demo in web browser - use VS Code tasks for background process
dev-demo:
    @echo "🎮 Use VS Code tasks for demo instead:"
    @echo "  1. Open Command Palette (Ctrl+Shift+P)"
    @echo "  2. Run 'Tasks: Run Task'"
    @echo "  3. Select 'Serve Demo'"
    @echo ""
    @echo "Or run manually:"
    @echo "  just build-demo"
    @echo "  just web-serve        # Then navigate to /demo.html"

# Start development environment - use VS Code tasks instead for multiple processes
dev:
    @echo "🎮 Use VS Code tasks for development instead:"
    @echo "  1. Open Command Palette (Ctrl+Shift+P)"
    @echo "  2. Run 'Tasks: Run Task'"
    @echo "  3. Select 'Start Development'"
    @echo ""
    @echo "Or run individual processes:"
    @echo "  just server           # Game server"
    @echo "  just watch-web        # Auto-rebuild WASM"
    @echo "  just web-serve        # Web server with live reload"

# Run two native clients for testing
test-multiplayer:
    #!/bin/bash
    echo "Starting multiplayer test..."
    cargo run --bin test_client Player1 &
    PID1=$!
    sleep 1
    cargo run --bin test_client Player2 &
    PID2=$!
    
    echo "Two clients running (PIDs: $PID1, $PID2)"
    echo "Press Enter to stop..."
    read
    kill $PID1 $PID2 2>/dev/null

# Check code quality
check:
    cargo check --workspace
    cargo clippy --workspace -- -D warnings

# Check if everything compiles without running
check-all:
    @echo "Checking all crates..."
    cargo check --workspace --all-targets
    @echo "Checking WASM build..."
    cd client && cargo check --target wasm32-unknown-unknown --no-default-features

# Check WASM build specifically
check-wasm:
    @echo "Checking WASM build..."
    cd client && cargo check --target wasm32-unknown-unknown --no-default-features

# Fix auto-fixable warnings
fix-warnings:
    @echo "Fixing auto-fixable warnings..."
    cargo fix --workspace --allow-dirty

# Run tests
test:
    cargo test --workspace

# Format code
fmt:
    cargo fmt --all

# Clean build artifacts
clean:
    cargo clean
    rm -rf dist/

# Full clean including logs
clean-all: clean
    rm -f server.log nohup.out
    rm -rf logs/

# Create release builds
release: release-server release-client

release-server:
    cargo build --bin server --release

release-client:
    cargo build --bin client --release

release-web:
    cd client && ./build-simple.sh --release
    npx browser-sync start --server dist --no-notify --port 8080

# Watch for changes and rebuild
watch:
    cargo watch -x "check --workspace"

# Generate documentation
doc:
    cargo doc --workspace --no-deps --open

# Update dependencies
update:
    cargo update

# Show project statistics
stats:
    @echo "Project Statistics:"
    @echo "=================="
    @find . -name "*.rs" -not -path "./target/*" | xargs wc -l | tail -1 | awk '{print "Rust LOC: " $1}'
    @echo -n "Crates: "
    @ls -1 */Cargo.toml | wc -l
    @echo -n "Dependencies: "
    @cargo tree | wc -l

# Check WASM target is installed
install-wasm-target:
    @rustup target add wasm32-unknown-unknown

# Quick start for development
quick-start: install-wasm-target dev

# CI/CD pipeline simulation
ci: fmt check-all test build

# Help with common issues
troubleshoot:
    @echo "Common troubleshooting steps:"
    @echo "============================"
    @echo "1. Port already in use:"
    @echo "   just kill-servers"
    @echo ""
    @echo "2. WASM build fails:"
    @echo "   rustup target add wasm32-unknown-unknown"
    @echo ""
    @echo "3. Server won't start:"
    @echo "   lsof -i :14191  # Check what's using the port"
    @echo ""
    @echo "4. Clean rebuild:"
    @echo "   just clean-all && just build"

# Kill all game-related processes
kill-servers:
    #!/bin/bash
    echo "Stopping all servers..."
    pkill -f "target/debug/server" || true
    pkill -f "trunk" || true
    pkill -f "test_client" || true
    pkill -f "browser-sync" || true
    pkill -f "bacon" || true
    # Also try to kill by port
    lsof -ti:8080 | xargs kill -9 2>/dev/null || true
    lsof -ti:14191 | xargs kill -9 2>/dev/null || true
    sleep 1
    echo "All servers stopped"

# Port check
check-ports:
    @echo "Checking ports..."
    @lsof -i :14191 || echo "Game server port (14191) is free"
    @lsof -i :8080 || echo "Web server port (8080) is free"
# Test AI system by adding AI players
test-ai:
    @echo "Testing AI system..."
    ./test_ai.sh

# Run the AI debug client
debug-ai:
    @echo "Starting AI debug client..."
    cargo run -p debug-client

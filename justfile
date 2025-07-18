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

# Start web development server
web-dev:
    #!/bin/bash
    just build-web
    cd dist && python3 -m http.server 8080

# Build and run the hybrid tile demo
build-demo:
    cd client && ./build-demo.sh

# Run demo in web browser
dev-demo:
    #!/bin/bash
    echo "Starting demo environment..."
    
    # Kill any existing web server
    pkill -f "python.*8080" || true
    sleep 1
    
    # Build demo WASM
    echo "Building demo..."
    cd client && ./build-demo.sh
    cd ..
    
    # Start web server
    echo "Starting web server..."
    cd dist && python -m http.server 8080 > /dev/null 2>&1 &
    WEB_PID=$!
    
    echo ""
    echo "🎮 Demo ready!"
    echo "Open: http://localhost:8080/demo.html"
    echo ""
    echo "Press Ctrl+C to stop"
    
    # Wait for interrupt
    trap "kill $WEB_PID 2>/dev/null; exit" INT
    wait

# Full development setup - server + web
dev:
    #!/bin/bash
    echo "Starting development environment..."
    
    # Kill any existing servers
    pkill -f "target/debug/server" || true
    pkill -f "python.*8080" || true
    sleep 1
    
    # Build WASM
    echo "Building WASM client..."
    cd client && ./build-simple.sh
    cd ..
    
    # Start game server
    echo "Starting game server..."
    RUST_LOG=info cargo run --bin server &
    SERVER_PID=$!
    
    # Wait for server to start
    sleep 2
    
    # Start web server
    echo "Starting web server..."
    cd dist && python3 -m http.server 8080 &
    WEB_PID=$!
    
    echo ""
    echo "🎮 Development environment ready!"
    echo "Game server: ws://127.0.0.1:14191/ws (PID: $SERVER_PID)"
    echo "Web server: http://localhost:8080 (PID: $WEB_PID)"
    echo ""
    echo "Press Ctrl+C to stop all servers"
    
    # Wait for interrupt
    trap "kill $SERVER_PID $WEB_PID 2>/dev/null; exit" INT
    wait

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

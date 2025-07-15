# Mech Battle Arena - Task Runner

# Default task - show available commands
default:
    @just --list

# Build all components
build: build-server build-client build-web

# Build the server
build-server:
    cargo build --bin server

# Build the native client
build-client:
    cargo build --bin client

# Build the WebAssembly client (debug mode)
build-web:
    #!/bin/bash
    echo "Building WebAssembly client (debug)..."
    mkdir -p web_build
    cd client && cargo build --target wasm32-unknown-unknown --no-default-features --features web
    cp ../target/wasm32-unknown-unknown/debug/client.wasm ../web_build/mech-battle-arena.wasm
    cd ..
    # Download miniquad JS bundle if not present
    if [ ! -f web_build/mq_js_bundle.js ]; then
        echo "Downloading miniquad JS bundle..."
        curl -o web_build/mq_js_bundle.js https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js
    fi
    # Ensure HTML exists
    if [ ! -f web_build/index.html ]; then
        ./build_web_macroquad.sh
    fi
    echo "WASM debug build complete!"

# Run the server
server:
    RUST_LOG=info cargo run --bin server

# Run the native client
client:
    RUST_LOG=info cargo run --bin client

# Run test client with custom name
test-client name="TestPlayer":
    cargo run --bin test_client {{name}}

# Start web server for WASM client
web-server:
    cd web_build && python3 serve.py

# Full development setup - server + web
dev: build-web
    #!/bin/bash
    echo "Starting development environment..."
    
    # Kill any existing servers
    pkill -f "target/debug/server" || true
    pkill -f "serve.py" || true
    sleep 1
    
    # Start game server
    echo "Starting game server..."
    RUST_LOG=info cargo run --bin server &
    SERVER_PID=$!
    
    # Wait for server to start
    sleep 2
    
    # Start web server
    echo "Starting web server..."
    cd web_build && python3 serve.py &
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

# Run tests
test:
    cargo test --workspace

# Format code
fmt:
    cargo fmt --all

# Clean build artifacts
clean:
    cargo clean
    rm -rf web_build/*.wasm

# Full clean including logs
clean-all: clean
    rm -f server.log nohup.out
    rm -rf web_build/web_server.log

# Create release builds
release: release-server release-client release-web

release-server:
    cargo build --bin server --release

release-client:
    cargo build --bin client --release

release-web:
    #!/bin/bash
    echo "Building WebAssembly client (release)..."
    mkdir -p web_build
    cd client && cargo build --target wasm32-unknown-unknown --release --no-default-features --features web
    cp ../target/wasm32-unknown-unknown/release/client.wasm ../web_build/mech-battle-arena.wasm
    cd ..
    # Download miniquad JS bundle if not present
    if [ ! -f web_build/mq_js_bundle.js ]; then
        echo "Downloading miniquad JS bundle..."
        curl -o web_build/mq_js_bundle.js https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js
    fi
    # Ensure HTML file exists
    if [ ! -f web_build/index.html ]; then
        ./build_web_macroquad.sh
    fi
    echo "Release WASM build complete! (1.3MB vs 23MB debug)"

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

# Initialize web build directory with HTML files
init-web:
    #!/bin/bash
    mkdir -p web_build
    echo "Web build directory initialized!"
    # HTML and Python files are already created by build_web.sh

# Quick start for development
quick-start: init-web build-web dev

# CI/CD pipeline simulation
ci: fmt check test build

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
    pkill -f "serve.py" || true
    pkill -f "python.*serve" || true
    pkill -f "python3.*8080" || true
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

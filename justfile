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
    npx browser-sync start --server dist --files "dist,dist/**/*" --no-notify --port 8080 --watch

# Start web server only (no build) - used by VS Code tasks
web-serve-only:
    npx browser-sync start --server dist --files "dist,dist/**/*" --no-notify --port 8080 --watch

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

# Watch and auto-rebuild documentation using bacon
watch-docs:
    bacon watch-docs

# Serve documentation files using browser-sync with live reload
serve-docs:
    npx browser-sync start --server target/doc --files "target/doc/**/*" --index "client/index.html" --no-notify --port 8081 --watch



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

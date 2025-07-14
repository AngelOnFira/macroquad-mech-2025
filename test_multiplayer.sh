#!/bin/bash

# Test script for multiplayer functionality
echo "Starting multiplayer test..."

# Kill any existing server
pkill -f "target/debug/server" || true
sleep 1

# Start server
echo "Starting server..."
RUST_LOG=info cargo run --bin server &
SERVER_PID=$!
sleep 2

# Start first client in background (headless)
echo "Starting client 1..."
RUST_LOG=info timeout 10 cargo run --bin client 2>&1 > client1.log &
CLIENT1_PID=$!

# Start second client in background (headless)  
echo "Starting client 2..."
sleep 1
RUST_LOG=info timeout 10 cargo run --bin client 2>&1 > client2.log &
CLIENT2_PID=$!

# Wait for clients to run
sleep 8

# Check if server is still running
if ps -p $SERVER_PID > /dev/null; then
    echo "✓ Server is running"
else
    echo "✗ Server crashed"
fi

# Check logs
echo -e "\nServer logs:"
tail -n 20 ~/.cargo/registry/src/*/env_logger*/src/lib.rs 2>/dev/null || echo "No server logs found"

echo -e "\nClient 1 logs:"
grep -E "(Connected|Joined|player)" client1.log | head -10

echo -e "\nClient 2 logs:"
grep -E "(Connected|Joined|player)" client2.log | head -10

# Cleanup
echo -e "\nCleaning up..."
kill $SERVER_PID 2>/dev/null || true
kill $CLIENT1_PID 2>/dev/null || true
kill $CLIENT2_PID 2>/dev/null || true
rm -f client1.log client2.log

echo "Test complete!"
#!/bin/bash

# AI Testing Session Script
# This script runs a comprehensive AI testing session with full logging

echo "=== Mech Battle Arena AI Testing Session ==="
echo "Starting at: $(date)"
echo ""

# Configuration
LOG_DIR="ai_test_logs"
SESSION_ID=$(date +%Y%m%d_%H%M%S)
SESSION_LOG_DIR="$LOG_DIR/session_$SESSION_ID"
MAIN_LOG="$SESSION_LOG_DIR/main.log"
SERVER_LOG="$SESSION_LOG_DIR/server.log"
AI_DECISIONS_LOG="$SESSION_LOG_DIR/ai_decisions.log"

# Create log directories
mkdir -p "$SESSION_LOG_DIR"

echo "Session ID: $SESSION_ID"
echo "Logs will be saved to: $SESSION_LOG_DIR"
echo ""

# Function to cleanup on exit
cleanup() {
    echo ""
    echo "Cleaning up..."
    pkill -f "target/debug/server" || true
    pkill -f "serve.py" || true
    echo "Test session ended at: $(date)"
}
trap cleanup EXIT

# Kill any existing servers
echo "Stopping any existing servers..."
pkill -f "target/debug/server" || true
pkill -f "serve.py" || true
sleep 2

# Start the server with detailed logging
echo "Starting game server with debug logging..."
RUST_LOG=debug cargo run -p server > "$SERVER_LOG" 2>&1 &
SERVER_PID=$!

# Wait for server to start
echo "Waiting for server to initialize..."
sleep 3

# Check if server is running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "ERROR: Server failed to start. Check $SERVER_LOG for details."
    exit 1
fi

echo "Server started successfully (PID: $SERVER_PID)"
echo ""

# Function to add AI players
add_ai_player() {
    local difficulty=$1
    local personality=$2
    echo "Adding AI: $personality (difficulty: $difficulty)..."
    
    response=$(curl -s -X POST http://localhost:14191/ai/add \
        -H "Content-Type: application/json" \
        -d "{\"difficulty\": $difficulty, \"personality\": \"$personality\"}" 2>&1)
    
    if [ $? -eq 0 ]; then
        echo "Response: $response"
        echo "$response" >> "$MAIN_LOG"
    else
        echo "Failed to add AI player"
    fi
}

# Add various AI players
echo "=== Adding AI Players ===" | tee -a "$MAIN_LOG"
echo ""

# Team 1: Balanced team
add_ai_player 0.7 "aggressive"
add_ai_player 0.6 "support"
add_ai_player 0.5 "balanced"

# Team 2: Mixed difficulties
add_ai_player 0.8 "defensive"
add_ai_player 0.4 "aggressive"
add_ai_player 0.6 "balanced"

echo ""
echo "=== AI Players Added ===" | tee -a "$MAIN_LOG"
echo ""

# Monitor the game for a period
MONITOR_DURATION=300  # 5 minutes
echo "Monitoring game for $MONITOR_DURATION seconds..."
echo "Tailing server log. Press Ctrl+C to stop early."
echo ""
echo "=== Server Log Output ===" | tee -a "$MAIN_LOG"

# Function to extract AI decisions from server log
extract_ai_info() {
    echo ""
    echo "=== Extracting AI Information ===" | tee -a "$MAIN_LOG"
    
    # Extract AI decisions
    grep -E "AI_|Decision|Hat:|Confidence:" "$SERVER_LOG" > "$AI_DECISIONS_LOG" || true
    
    # Count decisions by AI
    echo "Decision counts by AI:" | tee -a "$MAIN_LOG"
    grep -o "AI_[^[:space:]]*" "$SERVER_LOG" | sort | uniq -c | tee -a "$MAIN_LOG"
    
    # Extract errors
    echo "" | tee -a "$MAIN_LOG"
    echo "Errors found:" | tee -a "$MAIN_LOG"
    grep -i "error\|panic\|failed" "$SERVER_LOG" | tee -a "$MAIN_LOG" || echo "No errors found"
    
    # Extract warnings
    echo "" | tee -a "$MAIN_LOG"
    echo "Warnings found:" | tee -a "$MAIN_LOG"
    grep -i "warn" "$SERVER_LOG" | tee -a "$MAIN_LOG" || echo "No warnings found"
}

# Start monitoring with periodic status updates
START_TIME=$(date +%s)
while true; do
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))
    
    if [ $ELAPSED -ge $MONITOR_DURATION ]; then
        echo ""
        echo "Monitoring period complete."
        break
    fi
    
    # Show status every 30 seconds
    if [ $((ELAPSED % 30)) -eq 0 ] && [ $ELAPSED -gt 0 ]; then
        echo ""
        echo "=== Status Update at ${ELAPSED}s ===" | tee -a "$MAIN_LOG"
        
        # Check server health
        if kill -0 $SERVER_PID 2>/dev/null; then
            echo "Server: Running" | tee -a "$MAIN_LOG"
        else
            echo "Server: CRASHED!" | tee -a "$MAIN_LOG"
            break
        fi
        
        # Show recent AI activity
        echo "Recent AI activity:" | tee -a "$MAIN_LOG"
        tail -n 20 "$SERVER_LOG" | grep -E "AI_|Decision|Hat:" | tail -n 5 | tee -a "$MAIN_LOG" || true
    fi
    
    sleep 1
done

# Extract final information
extract_ai_info

# Generate summary report
echo ""
echo "=== Test Session Summary ===" | tee -a "$MAIN_LOG"
echo "Duration: $ELAPSED seconds" | tee -a "$MAIN_LOG"
echo "Log files saved to: $SESSION_LOG_DIR" | tee -a "$MAIN_LOG"
echo "" | tee -a "$MAIN_LOG"
echo "File sizes:" | tee -a "$MAIN_LOG"
ls -lh "$SESSION_LOG_DIR"/* | tee -a "$MAIN_LOG"

echo ""
echo "Test session complete. Logs saved to: $SESSION_LOG_DIR"
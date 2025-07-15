#!/bin/bash

# Test script to add AI players to the game

echo "Testing AI addition to Mech Battle Arena"
echo "========================================"

# Start the server in background if not already running
if ! lsof -i:14191 >/dev/null 2>&1; then
    echo "Starting game server..."
    cargo run -p server > server.log 2>&1 &
    SERVER_PID=$!
    sleep 2
else
    echo "Server already running on port 14191"
fi

# Add AI players with different personalities and difficulties
echo ""
echo "Adding AI players..."
echo ""

# Add balanced AI with medium difficulty
echo "1. Adding Balanced AI (difficulty: 0.5)..."
curl -X POST http://localhost:14191/ai/add \
  -H "Content-Type: application/json" \
  -d '{"difficulty": 0.5, "personality": "balanced"}' \
  | jq .

echo ""

# Add aggressive AI with high difficulty
echo "2. Adding Aggressive AI (difficulty: 0.9)..."
curl -X POST http://localhost:14191/ai/add \
  -H "Content-Type: application/json" \
  -d '{"difficulty": 0.9, "personality": "aggressive"}' \
  | jq .

echo ""

# Add defensive AI with low difficulty
echo "3. Adding Defensive AI (difficulty: 0.3)..."
curl -X POST http://localhost:14191/ai/add \
  -H "Content-Type: application/json" \
  -d '{"difficulty": 0.3, "personality": "defensive"}' \
  | jq .

echo ""

# Add support AI with medium difficulty
echo "4. Adding Support AI (difficulty: 0.6)..."
curl -X POST http://localhost:14191/ai/add \
  -H "Content-Type: application/json" \
  -d '{"difficulty": 0.6, "personality": "support"}' \
  | jq .

echo ""
echo "========================================"
echo "AI players added! Check server logs for AI activity."
echo ""
echo "To view server logs: tail -f server.log"
echo "To stop server: kill $SERVER_PID"
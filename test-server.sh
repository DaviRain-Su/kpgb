#!/bin/bash
# Test script to verify server links

echo "Starting server test..."

# Start server in background
cargo run -- serve --port 3002 > /dev/null 2>&1 &
SERVER_PID=$!

# Wait for server to start
sleep 3

echo "Testing home page links..."
curl -s http://localhost:3002/ | grep -o 'href="[^"]*"' | grep -E "(archive|feed|kpgb)" | head -10

echo -e "\nTesting archive page..."
curl -s http://localhost:3002/archive | grep -o 'href="[^"]*"' | head -5

echo -e "\nTesting if /kpgb paths exist in links..."
if curl -s http://localhost:3002/ | grep -q 'href="/kpgb'; then
    echo "WARNING: Found /kpgb in links!"
else
    echo "SUCCESS: No /kpgb prefix in links"
fi

# Kill server
kill $SERVER_PID 2>/dev/null

echo "Test complete."
#!/bin/bash
# Test WebSocket Deadlock Fix
# This script helps verify that the broadcast messages are being sent continuously

echo "🧪 Testing WebSocket Real-time Updates..."
echo ""
echo "Starting server in background..."

# Start server and capture output
cd /home/thichuong/Desktop/Web-server-Report
cargo run 2>&1 | grep -E "📊 Dashboard data broadcasted|📡 Broadcast message sent|WebSocket" &
SERVER_PID=$!

echo "Server PID: $SERVER_PID"
echo ""
echo "⏳ Waiting 5 seconds for server to start..."
sleep 5

echo ""
echo "✅ Server should be running at http://localhost:8000"
echo ""
echo "📋 Testing Instructions:"
echo "  1. Open http://localhost:8000 in your browser"
echo "  2. Open Developer Console (F12)"
echo "  3. Watch for messages every 2 seconds:"
echo "     📨 [HH:MM:SS] WebSocket message type: dashboard_update"
echo "     📊 [HH:MM:SS] Market data update received - processing..."
echo "     ✅ Updated BTC: \$XXXXX.XX (±X.XX%)"
echo ""
echo "🔍 What to verify:"
echo "  ✅ Messages arrive every 2 seconds (not in bursts)"
echo "  ✅ Timestamps show consistent intervals"
echo "  ✅ Prices update smoothly on screen"
echo ""
echo "Press Ctrl+C to stop the server when done testing"
echo ""

# Wait for user to stop
wait $SERVER_PID

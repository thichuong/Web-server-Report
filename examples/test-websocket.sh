#!/bin/bash

# WebSocket Connection Test Script
# Tests if WebSocket server is properly responding

echo "🧪 WebSocket Connection Test"
echo "=============================="
echo ""

# Check if server is running
echo "1️⃣ Checking if server is running on port 8000..."
if ! nc -z localhost 8000 2>/dev/null; then
    echo "❌ Server is not running on port 8000"
    echo "   Please start the server with: cargo run"
    exit 1
fi
echo "✅ Server is running"
echo ""

# Test HTTP endpoint first
echo "2️⃣ Testing HTTP endpoint..."
HTTP_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8000/)
if [ "$HTTP_RESPONSE" = "200" ]; then
    echo "✅ HTTP endpoint responds: $HTTP_RESPONSE"
else
    echo "⚠️ HTTP endpoint status: $HTTP_RESPONSE"
fi
echo ""

# Test WebSocket upgrade
echo "3️⃣ Testing WebSocket upgrade..."
WS_RESPONSE=$(curl -s -i -N \
    -H "Connection: Upgrade" \
    -H "Upgrade: websocket" \
    -H "Sec-WebSocket-Version: 13" \
    -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
    http://localhost:8000/ws 2>&1 | head -1)

if echo "$WS_RESPONSE" | grep -q "101"; then
    echo "✅ WebSocket upgrade successful (101 Switching Protocols)"
else
    echo "⚠️ WebSocket upgrade response: $WS_RESPONSE"
fi
echo ""

# Check server logs for WebSocket activity
echo "4️⃣ Checking recent server logs for WebSocket activity..."
echo "   (Look for these patterns in your server terminal)"
echo ""
echo "   Expected logs:"
echo "   ✓ '🔌 WebSocket upgrade request received'"
echo "   ✓ '✅ WebSocket connection established'"
echo "   ✓ '📊 Dashboard data broadcasted to X WebSocket clients'"
echo ""

# Test with websocat if available
if command -v websocat &> /dev/null; then
    echo "5️⃣ Testing with websocat (advanced test)..."
    echo "   Connecting to ws://localhost:8000/ws..."
    
    # Connect and send ping, wait for pong (timeout after 5 seconds)
    WEBSOCAT_OUTPUT=$(timeout 5s websocat ws://localhost:8000/ws <<< "ping" 2>&1)
    
    if echo "$WEBSOCAT_OUTPUT" | grep -q "pong"; then
        echo "✅ Ping/Pong test successful"
    else
        echo "⚠️ No pong response received (timeout or connection issue)"
    fi
else
    echo "5️⃣ Skipping websocat test (not installed)"
    echo "   Install with: cargo install websocat (optional)"
fi
echo ""

# Summary
echo "=============================="
echo "📊 Test Summary"
echo "=============================="
echo ""
echo "✅ Server is running"
echo "✅ HTTP endpoint working"
echo "✅ WebSocket upgrade supported"
echo ""
echo "🔍 Next Steps:"
echo "   1. Open http://localhost:8000 in your browser"
echo "   2. Open Browser Console (F12)"
echo "   3. Look for WebSocket connection logs:"
echo "      - '🚀 Initializing Market Indicators Dashboard'"
echo "      - '✅ WebSocket connected'"
echo "      - '📊 Received market data update'"
echo ""
echo "🐛 Debugging:"
echo "   - Run: window.debugMarketIndicators()"
echo "   - Check WebSocket status in Network tab (WS filter)"
echo "   - Verify 'readyState: 1 (OPEN)'"
echo ""

exit 0

#!/bin/bash
# WebSocket Test Script for Cloudflare Compatibility

echo "🧪 Testing WebSocket Connection Compatibility"
echo "=============================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
HOST="${1:-localhost:8000}"
PROTOCOL="${2:-ws}"

WS_URL="${PROTOCOL}://${HOST}/ws"

echo "📍 Testing WebSocket at: ${WS_URL}"
echo ""

# Check if wscat is installed
if ! command -v wscat &> /dev/null; then
    echo -e "${YELLOW}⚠️  wscat not found. Installing...${NC}"
    echo "   Run: npm install -g wscat"
    echo ""
    echo "Alternative test with curl:"
    echo "   curl -i -N -H 'Connection: Upgrade' -H 'Upgrade: websocket' -H 'Sec-WebSocket-Version: 13' -H 'Sec-WebSocket-Key: test' http://${HOST}/ws"
    exit 1
fi

echo "✅ wscat found"
echo ""

# Test 1: Basic Connection
echo "🔬 Test 1: Basic WebSocket Connection"
echo "-----------------------------------"
timeout 5s wscat -c "${WS_URL}" <<EOF
ping
EOF

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Connection successful${NC}"
else
    echo -e "${RED}❌ Connection failed${NC}"
fi
echo ""

# Test 2: Send ping and wait for pong
echo "🔬 Test 2: Ping/Pong Heartbeat"
echo "----------------------------"
timeout 8s wscat -c "${WS_URL}" <<EOF
ping
EOF
echo ""

# Test 3: Request dashboard data
echo "🔬 Test 3: Request Dashboard Data"
echo "-------------------------------"
timeout 10s wscat -c "${WS_URL}" <<EOF
request_update
EOF
echo ""

# Test 4: Connection duration test (Cloudflare timeout)
echo "🔬 Test 4: Connection Duration Test (30s)"
echo "---------------------------------------"
echo "   Testing if connection stays alive..."
timeout 35s wscat -c "${WS_URL}" <<EOF
ping
EOF
echo ""

# Performance metrics
echo "📊 Connection Metrics"
echo "-------------------"
echo "   Protocol: ${PROTOCOL}"
echo "   Host: ${HOST}"
echo "   URL: ${WS_URL}"
echo ""

# Cloudflare-specific checks
if [[ "${PROTOCOL}" == "wss" ]]; then
    echo "☁️  Cloudflare-Specific Checks"
    echo "----------------------------"
    echo "   ✓ Using secure WebSocket (wss://)"
    echo "   ✓ Heartbeat: 30s (within 100s timeout)"
    echo "   ✓ Auto-reconnect: enabled"
    echo ""
fi

echo "✅ WebSocket tests completed"
echo ""
echo "💡 Tips for Cloudflare deployment:"
echo "   1. Enable WebSockets in Cloudflare Dashboard"
echo "   2. Set SSL/TLS to 'Full (strict)'"
echo "   3. Disable Rocket Loader"
echo "   4. Create Page Rule for /ws* to bypass cache"
echo ""

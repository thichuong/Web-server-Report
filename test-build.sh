#!/bin/bash

echo "🧪 Quick Railway Build Test"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}Testing different Dockerfile approaches...${NC}"

# Test 1: Simple build (most reliable)
echo -e "\n📦 ${YELLOW}Test 1: Simple single-stage build${NC}"
if timeout 300s docker build -f Dockerfile.simple-railway -t railway-test-simple . --quiet; then
    echo -e "✅ ${GREEN}Simple build: SUCCESS${NC}"
    SIMPLE_SIZE=$(docker images railway-test-simple --format "table {{.Size}}" | tail -n 1)
    echo -e "📏 Image size: ${SIMPLE_SIZE}"
else
    echo -e "❌ ${RED}Simple build: FAILED${NC}"
fi

# Test 2: Multi-stage build (smaller but complex)
echo -e "\n📦 ${YELLOW}Test 2: Multi-stage build${NC}"
if timeout 300s docker build -f Dockerfile.railway -t railway-test-multi . --quiet; then
    echo -e "✅ ${GREEN}Multi-stage build: SUCCESS${NC}"
    MULTI_SIZE=$(docker images railway-test-multi --format "table {{.Size}}" | tail -n 1)
    echo -e "📏 Image size: ${MULTI_SIZE}"
else
    echo -e "❌ ${RED}Multi-stage build: FAILED${NC}"
fi

# Test 3: Check what works
echo -e "\n🔍 ${YELLOW}Recommendation based on results:${NC}"

if docker images railway-test-simple &>/dev/null; then
    echo -e "✅ Use Dockerfile.simple-railway for Railway deployment"
    echo -e "📝 Update railway.json to use this file"
else
    echo -e "⚠️ Try optimized build or check network issues"
fi

# Cleanup
echo -e "\n🧹 Cleaning up test images..."
docker rmi railway-test-simple railway-test-multi 2>/dev/null || true

echo -e "\n🎯 ${GREEN}Test completed!${NC}"

#!/bin/bash

echo "🚀 Preparing Railway Deployment..."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Run this script from the Web-server-Report directory${NC}"
    exit 1
fi

echo "📋 Pre-deployment checklist:"

# 1. Check Rust toolchain
echo -n "🦀 Checking Rust toolchain... "
if command -v cargo &> /dev/null; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌ Cargo not found${NC}"
    exit 1
fi

# 2. Test local build
echo -n "🔨 Testing release build... "
if cargo build --release --quiet; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

# 3. Check static assets
echo -n "📁 Checking static assets... "
MISSING_ASSETS=0

for dir in "dashboards" "shared_components" "shared_assets"; do
    if [ ! -d "$dir" ]; then
        echo -e "\n${RED}❌ Missing directory: $dir${NC}"
        MISSING_ASSETS=1
    fi
done

if [ $MISSING_ASSETS -eq 0 ]; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌ Missing static assets${NC}"
    exit 1
fi

# 4. Check Railway config files
echo -n "⚙️ Checking Railway configuration... "
CONFIG_OK=1

if [ ! -f "railway.json" ]; then
    echo -e "\n${RED}❌ Missing railway.json${NC}"
    CONFIG_OK=0
fi

if [ ! -f "Dockerfile.railway" ]; then
    echo -e "\n${RED}❌ Missing Dockerfile.railway${NC}"
    CONFIG_OK=0
fi

if [ ! -f "Procfile" ]; then
    echo -e "\n${RED}❌ Missing Procfile${NC}"
    CONFIG_OK=0
fi

if [ $CONFIG_OK -eq 1 ]; then
    echo -e "${GREEN}✅${NC}"
else
    exit 1
fi

# 5. Check environment template
echo -n "🌍 Checking environment template... "
if [ -f ".env.example" ]; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌ Missing .env.example${NC}"
    exit 1
fi

# 6. Display configuration summary
echo -e "\n📊 ${YELLOW}Configuration Summary:${NC}"
echo "📦 Docker file: Dockerfile.railway"
echo "🚀 Start command: $(grep startCommand railway.json | cut -d'"' -f4)"
echo "🌐 Health check: $(grep healthcheckPath railway.json | cut -d'"' -f4)"
echo "📝 Process file: $(cat Procfile)"

echo -e "\n⚠️ ${YELLOW}Before deploying to Railway:${NC}"
echo "1. Set up environment variables in Railway dashboard:"
echo "   - DATABASE_URL"
echo "   - REDIS_URL" 
echo "   - TAAPI_SECRET"
echo "   - FINNHUB_API_KEY"
echo "   - AUTO_UPDATE_SECRET_KEY"
echo "2. Make sure Railway project is connected to this repo"
echo "3. Deploy using: railway up or git push to trigger auto-deploy"

echo -e "\n🎯 ${YELLOW}Static Assets Check:${NC}"
find dashboards -name "*.html" -o -name "*.css" -o -name "*.js" | head -5 | sed 's/^/   ✓ /'
find shared_components -name "*.html" -o -name "*.css" -o -name "*.js" | head -5 | sed 's/^/   ✓ /'
find shared_assets -name "*.css" -o -name "*.js" | head -5 | sed 's/^/   ✓ /'

echo -e "\n${GREEN}✅ Pre-deployment check completed successfully!${NC}"
echo -e "${GREEN}🚀 Ready for Railway deployment!${NC}"

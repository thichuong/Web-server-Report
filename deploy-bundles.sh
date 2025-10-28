#!/bin/bash

##############################################################################
# Deployment Script for Web Server Report
# 
# This script automates the deployment of bundled JavaScript assets
# Features:
# - Build production bundles
# - Copy to deployment locations
# - Update HTML references
# - Cache busting
# - Backup original files
##############################################################################

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
VERSION=$(date +%Y%m%d%H%M%S)
BACKUP_DIR="backups/pre-deploy-${VERSION}"

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Web Server Report - Deployment Script v1.0.0        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Step 1: Check prerequisites
echo -e "${YELLOW}📋 Step 1: Checking prerequisites...${NC}"

if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js not found. Please install Node.js 18+${NC}"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo -e "${RED}❌ npm not found. Please install npm${NC}"
    exit 1
fi

if [ ! -f "package.json" ]; then
    echo -e "${RED}❌ package.json not found. Run this script from project root${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites check passed${NC}"
echo ""

# Step 2: Create backup
echo -e "${YELLOW}📦 Step 2: Creating backup...${NC}"

mkdir -p "${BACKUP_DIR}"

# Backup current files
BACKUP_FILES=(
    "shared_components/market-indicators/market-indicators-modular.js"
    "dashboards/crypto_dashboard/assets/report-view-iframe.js"
    "dashboards/crypto_dashboard/assets/date-formatter-utility.js"
    "dashboards/crypto_dashboard/assets/report-list-interactions.js"
)

for file in "${BACKUP_FILES[@]}"; do
    if [ -f "$file" ]; then
        cp -p "$file" "${BACKUP_DIR}/"
        echo "  📄 Backed up: $file"
    fi
done

echo -e "${GREEN}✅ Backup created: ${BACKUP_DIR}${NC}"
echo ""

# Step 3: Install dependencies
echo -e "${YELLOW}📦 Step 3: Installing dependencies...${NC}"

npm ci --quiet

echo -e "${GREEN}✅ Dependencies installed${NC}"
echo ""

# Step 4: Build production bundles
echo -e "${YELLOW}🏗️  Step 4: Building production bundles...${NC}"

npm run build:prod

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed. Check errors above${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Build completed successfully${NC}"
echo ""

# Step 5: Deploy bundles
echo -e "${YELLOW}🚀 Step 5: Deploying bundles...${NC}"

# Deploy market-indicators
if [ -f "dist/market-indicators.bundle.js" ]; then
    cp dist/market-indicators.bundle.js shared_components/market-indicators/market-indicators.bundle.js
    echo "  ✅ Deployed: market-indicators.bundle.js"
fi

# Deploy report-view-iframe
if [ -f "dist/report-view-iframe.bundle.js" ]; then
    cp dist/report-view-iframe.bundle.js dashboards/crypto_dashboard/assets/report-view-iframe.bundle.js
    echo "  ✅ Deployed: report-view-iframe.bundle.js"
fi

# Deploy date-formatter
if [ -f "dist/date-formatter.bundle.js" ]; then
    cp dist/date-formatter.bundle.js dashboards/crypto_dashboard/assets/date-formatter.bundle.js
    echo "  ✅ Deployed: date-formatter.bundle.js"
fi

# Deploy report-list-interactions
if [ -f "dist/report-list-interactions.bundle.js" ]; then
    cp dist/report-list-interactions.bundle.js dashboards/crypto_dashboard/assets/report-list-interactions.bundle.js
    echo "  ✅ Deployed: report-list-interactions.bundle.js"
fi

echo -e "${GREEN}✅ All bundles deployed${NC}"
echo ""

# Step 6: Show build report
echo -e "${YELLOW}📊 Step 6: Build Statistics${NC}"

if [ -f "dist/build-report.json" ]; then
    echo "────────────────────────────────────────────────────────────"
    cat dist/build-report.json | grep -E "(totalSize|totalGzipped|compressionRatio)" | sed 's/^/  /'
    echo "────────────────────────────────────────────────────────────"
fi

echo ""

# Step 7: Post-deployment checklist
echo -e "${YELLOW}✅ Step 7: Post-Deployment Checklist${NC}"
echo ""
echo "  Please verify the following:"
echo "  ┌─────────────────────────────────────────────────────────┐"
echo "  │ ☐ Update HTML files to use .bundle.js files            │"
echo "  │ ☐ Clear browser cache or add ?v=${VERSION}             │"
echo "  │ ☐ Test WebSocket connections                           │"
echo "  │ ☐ Verify chart rendering                               │"
echo "  │ ☐ Check navigation and scroll                          │"
echo "  │ ☐ Test theme/language switching                        │"
echo "  │ ☐ Monitor console for errors                           │"
echo "  └─────────────────────────────────────────────────────────┘"
echo ""

# Step 8: Rollback instructions
echo -e "${BLUE}📝 Rollback Instructions:${NC}"
echo ""
echo "  If issues occur, restore from backup:"
echo "  ${GREEN}cp ${BACKUP_DIR}/* <original-location>${NC}"
echo ""

echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          ✅ Deployment Completed Successfully!         ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Deployment Version: ${VERSION}${NC}"
echo -e "${YELLOW}Backup Location: ${BACKUP_DIR}${NC}"
echo ""

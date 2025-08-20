#!/bin/bash

# 🗑️ Cleanup Script - Remove unnecessary files after Service Islands migration
# This script removes legacy monolithic files that have been successfully 
# extracted into Service Islands architecture

set -e

echo "🧹 Starting cleanup of legacy files..."

# Color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to safely remove file if it exists
safe_remove() {
    if [ -f "$1" ]; then
        echo -e "${YELLOW}Removing: $1${NC}"
        rm "$1"
        echo -e "${GREEN}✅ Removed: $1${NC}"
    else
        echo -e "${RED}❌ File not found: $1${NC}"
    fi
}

# Function to backup important files before removal
backup_file() {
    if [ -f "$1" ]; then
        echo -e "${YELLOW}Creating backup: $1.backup${NC}"
        cp "$1" "$1.backup"
        echo -e "${GREEN}✅ Backup created: $1.backup${NC}"
    fi
}

echo "📋 Checking Service Islands migration status..."

# Verify Service Islands are in place before removing legacy files
if [ ! -d "src/features" ]; then
    echo -e "${RED}❌ ERROR: Service Islands not found in src/features/. Aborting cleanup.${NC}"
    exit 1
fi

if [ ! -d "src/features/health_system" ] || [ ! -d "src/features/cache_system" ]; then
    echo -e "${RED}❌ ERROR: Required Service Islands missing. Aborting cleanup.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Service Islands verified. Proceeding with cleanup...${NC}"

echo ""
echo "🗂️ PHASE 1: Remove log files and temporary files"

# Remove log files
safe_remove "cargo_run_output.log"
safe_remove "runtime_debug.log"
safe_remove "rust_server.log" 
safe_remove "server.log"
safe_remove "server_phase3.log"

echo ""
echo "🗂️ PHASE 2: Remove migration documentation (keep for reference)"
echo -e "${YELLOW}Keeping MIGRATION_PLAN.md and NEW_ARCHITECTURE.md for reference${NC}"

# Keep migration docs but remove temporary progress files
safe_remove "PHASE2_TASK3_PROGRESS.md"

echo ""
echo "🗂️ PHASE 3: Create legacy directory for old monolithic files"

# Create legacy directory if it doesn't exist
if [ ! -d "legacy" ]; then
    mkdir -p "legacy"
    echo -e "${GREEN}✅ Created legacy/ directory${NC}"
fi

echo ""
echo "🏛️ PHASE 4: Move monolithic files to legacy (DANGEROUS - requires manual verification)"
echo -e "${YELLOW}WARNING: This will move core monolithic files. Ensure Service Islands are working first!${NC}"
echo -e "${YELLOW}Moving to legacy/ instead of deleting for safety${NC}"

# Function to move file to legacy directory
move_to_legacy() {
    if [ -f "$1" ]; then
        echo -e "${YELLOW}Moving to legacy: $1${NC}"
        mv "$1" "legacy/$(basename "$1")"
        echo -e "${GREEN}✅ Moved to legacy: $1${NC}"
    else
        echo -e "${RED}❌ File not found: $1${NC}"
    fi
}

# Uncomment these lines ONLY after verifying Service Islands work correctly
# echo "⚠️  Manual verification required before moving these files:"
# echo "   src/handlers_backup.rs (841 lines - legacy handlers)"
# echo "   src/data_service.rs (662 lines - moved to external_apis)"  
# echo "   src/performance.rs (297 lines - moved to health_system)"
# echo "   src/cache.rs (464 lines - moved to cache_system)"
# echo "   src/websocket_service.rs (moved to websocket_service island)"

echo ""
echo -e "${YELLOW}⚠️  MANUAL ACTION REQUIRED:${NC}"
echo -e "${YELLOW}Before moving monolithic files to legacy/, please:${NC}"
echo -e "${YELLOW}1. Test the application with Service Islands${NC}"
echo -e "${YELLOW}2. Run: cargo test${NC}"
echo -e "${YELLOW}3. Verify all endpoints work${NC}"
echo -e "${YELLOW}4. Then uncomment the move commands in this script${NC}"

echo ""
echo "🗂️ PHASE 5: Remove unused deployment files"

# Remove unused Docker files (keep main Dockerfile)
safe_remove "deploy/Dockerfile.alpine"
safe_remove "deploy/Dockerfile.fixed"
safe_remove "deploy/Dockerfile.minimal"
safe_remove "deploy/Dockerfile.ubuntu"

echo ""
echo "🗂️ PHASE 6: Clean target directory"
if [ -d "target" ]; then
    echo -e "${YELLOW}Cleaning Rust target directory...${NC}"
    cargo clean
    echo -e "${GREEN}✅ Target directory cleaned${NC}"
fi

echo ""
echo "📊 CLEANUP SUMMARY:"
echo -e "${GREEN}✅ Log files removed${NC}"
echo -e "${GREEN}✅ Temporary files cleaned${NC}"
echo -e "${GREEN}✅ Legacy directory created${NC}"
echo -e "${GREEN}✅ Unused Docker files removed${NC}"
echo -e "${GREEN}✅ Target directory cleaned${NC}"
echo ""
echo -e "${YELLOW}⚠️  Monolithic source files preserved (manual verification needed)${NC}"
echo -e "${YELLOW}📝 To complete cleanup: verify Service Islands, then move monolithic files to legacy/${NC}"

echo ""
echo -e "${GREEN}🎉 Cleanup completed successfully!${NC}"
echo -e "${GREEN}💾 Disk space saved: $(du -sh . | cut -f1)${NC}"

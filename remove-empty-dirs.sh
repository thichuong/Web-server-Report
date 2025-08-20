#!/bin/bash

# 🗑️ Remove Empty Directories Script
# Safely removes empty directories from the Service Islands migration

set -e

echo "🧹 Removing empty directories from Service Islands migration..."

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

removed_count=0

# Function to safely remove empty directory
remove_empty_dir() {
    if [ -d "$1" ] && [ -z "$(ls -A "$1" 2>/dev/null)" ]; then
        echo -e "${YELLOW}Removing empty directory: $1${NC}"
        rmdir "$1"
        echo -e "${GREEN}✅ Removed: $1${NC}"
        ((removed_count++))
    elif [ -d "$1" ]; then
        echo -e "${RED}❌ Directory not empty: $1${NC}"
    else
        echo -e "${RED}❌ Directory not found: $1${NC}"
    fi
}

echo ""
echo "🗂️ PHASE 1: Remove empty Service Islands placeholder directories"

# Service Islands - crypto_reports (pending extraction)
remove_empty_dir "src/features/crypto_reports/handlers"
remove_empty_dir "src/features/crypto_reports/models"  
remove_empty_dir "src/features/crypto_reports/services"

# Service Islands - dashboard (pending extraction)
remove_empty_dir "src/features/dashboard/handlers"
remove_empty_dir "src/features/dashboard/models"
remove_empty_dir "src/features/dashboard/services"

# Service Islands - health_system (placeholder dirs)
remove_empty_dir "src/features/health_system/handlers"
remove_empty_dir "src/features/health_system/models"

# Service Islands - cache_system (placeholder dirs)
remove_empty_dir "src/features/cache_system/services"
remove_empty_dir "src/features/cache_system/models"

# Service Islands - websocket_service (placeholder dirs)  
remove_empty_dir "src/features/websocket_service/services"
remove_empty_dir "src/features/websocket_service/handlers"

# Service Islands - external_apis (placeholder dirs)
remove_empty_dir "src/features/external_apis/services"
remove_empty_dir "src/features/external_apis/models"

echo ""
echo "🗂️ PHASE 2: Remove empty monolithic architecture directories"

# Old monolithic structure
remove_empty_dir "src/infrastructure/di"
remove_empty_dir "src/infrastructure/database"
remove_empty_dir "src/presentation/http"
remove_empty_dir "src/presentation/routes"
remove_empty_dir "src/presentation/middleware"
remove_empty_dir "src/domain/repositories"
remove_empty_dir "src/application/services"
remove_empty_dir "src/core"
remove_empty_dir "src/services/crypto_reports"
remove_empty_dir "src/services/dashboard_api"
remove_empty_dir "src/utils"

echo ""
echo "🗂️ PHASE 3: Remove empty parent directories"

# Remove parent directories if they become empty
remove_empty_dir "src/infrastructure"
remove_empty_dir "src/presentation" 
remove_empty_dir "src/domain"
remove_empty_dir "src/application"
remove_empty_dir "src/services"

# Remove legacy directory if empty
remove_empty_dir "legacy"

echo ""
echo "🗂️ PHASE 4: Clean up any remaining empty directories"

# Find and remove any remaining empty directories
echo -e "${YELLOW}Searching for any remaining empty directories...${NC}"

empty_dirs=$(find . -type d -empty -not -path "./.git/*" -not -path "./target/*" 2>/dev/null || true)

if [ -n "$empty_dirs" ]; then
    echo -e "${YELLOW}Found additional empty directories:${NC}"
    echo "$empty_dirs"
    
    # Remove them
    echo "$empty_dirs" | while IFS= read -r dir; do
        if [ -n "$dir" ]; then
            remove_empty_dir "$dir"
        fi
    done
else
    echo -e "${GREEN}✅ No additional empty directories found${NC}"
fi

echo ""
echo "📊 CLEANUP SUMMARY"
echo -e "${GREEN}✅ Total directories removed: $removed_count${NC}"

# Verify current directory structure
echo ""
echo "📂 CURRENT SERVICE ISLANDS STRUCTURE:"
echo -e "${YELLOW}Active Service Islands:${NC}"
ls -la src/features/ 2>/dev/null | grep '^d' | grep -v '^\.$' | grep -v '^\.\.$' || echo "No features directory found"

echo ""
echo "📂 REMAINING SRC STRUCTURE:"  
find src -type d -maxdepth 2 | sort

echo ""
echo -e "${GREEN}🎉 Empty directory cleanup completed!${NC}"
echo -e "${GREEN}📁 Repository structure optimized for Service Islands architecture${NC}"

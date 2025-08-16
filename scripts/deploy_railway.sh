#!/bin/bash

# Railway Deployment Script
# Chuẩn bị và deploy server lên Railway

set -e

echo "🚂 RAILWAY DEPLOYMENT PREPARATION"
echo "=================================="

# 1. Kiểm tra Railway CLI
if ! command -v railway &> /dev/null; then
    echo "❌ Railway CLI not found. Installing..."
    curl -fsSL https://railway.app/install.sh | sh
    echo "✅ Railway CLI installed"
fi

# 2. Kiểm tra login status
echo "🔐 Checking Railway login status..."
if ! railway whoami &> /dev/null; then
    echo "⚠️  Not logged in to Railway. Please login:"
    railway login
fi

# 3. Kiểm tra các files cần thiết
echo "📁 Checking required files..."

required_files=(
    "Dockerfile.railway"
    "railway.json"
    "nixpacks.toml"
    "Cargo.toml"
    "src/main.rs"
    "dashboards"
    "shared_components"
    "shared_assets"
)

for file in "${required_files[@]}"; do
    if [[ ! -e "$file" ]]; then
        echo "❌ Missing required file/directory: $file"
        exit 1
    fi
done
echo "✅ All required files present"

# 4. Build test locally (SSL-optimized)
echo "🔨 Testing local build with SSL optimizations..."
if docker build -f Dockerfile.railway -t web-server-railway-test . > /dev/null 2>&1; then
    echo "✅ Docker build successful (with SSL/TLS support)"
    docker rmi web-server-railway-test 2>/dev/null || true
else
    echo "❌ Docker build failed. Please fix before deploying."
    exit 1
fi

# 5. Kiểm tra environment variables
echo "🔧 Environment variables check..."
echo "Required environment variables for Railway:"
echo "  - DATABASE_URL (PostgreSQL connection)"
echo "  - TAAPI_SECRET (API key)"
echo "  - REDIS_URL (Redis connection, optional)"
echo ""
echo "Make sure to set these in Railway dashboard before deployment!"

# 6. Deploy options
echo "🚀 Deployment options:"
echo "1. Deploy to Railway (if project exists)"
echo "2. Create new Railway project and deploy"
echo "3. Just validate configuration"
echo ""

read -p "Choose option (1-3): " option

case $option in
    1)
        echo "🚂 Deploying to existing Railway project..."
        railway up
        ;;
    2)
        echo "🆕 Creating new Railway project..."
        railway login
        railway up --detach
        ;;
    3)
        echo "✅ Configuration validation complete"
        ;;
    *)
        echo "❌ Invalid option"
        exit 1
        ;;
esac

echo ""
echo "📋 POST-DEPLOYMENT CHECKLIST:"
echo "=============================="
echo "1. ✅ Set DATABASE_URL in Railway dashboard"
echo "2. ✅ Set TAAPI_SECRET in Railway dashboard" 
echo "3. ✅ Set REDIS_URL (optional) in Railway dashboard"
echo "4. ✅ Verify health check: https://<your-domain>/health"
echo "5. ✅ Test WebSocket: wss://<your-domain>/ws"
echo "6. ✅ Monitor logs: railway logs"
echo ""
echo "🎉 Deployment preparation complete!"
echo "Your optimized Rust server should be running on Railway!"

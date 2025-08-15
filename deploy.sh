#!/bin/bash

# Railway Deployment Script for Rust Web Server
# This script helps deploy your Rust application to Railway

echo "🚀 Railway Deployment Setup for Rust Web Server"
echo "================================================"

# Check if we're in the correct directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

echo "✅ Project structure verified"

# Check if Dockerfile exists
if [ ! -f "Dockerfile" ]; then
    echo "❌ Error: Dockerfile not found. Please ensure Dockerfile is created."
    exit 1
fi

echo "✅ Dockerfile found (Fixed Docker build issues)"

# Test local build
echo "🔧 Testing local build..."
if cargo build --release > /dev/null 2>&1; then
    echo "✅ Local build successful"
else
    echo "❌ Local build failed. Please fix compilation errors first."
    exit 1
fi

# Check if railway.json exists
if [ ! -f "railway.json" ]; then
    echo "❌ Error: railway.json not found. Please ensure railway.json is created."
    exit 1
fi

echo "✅ Railway configuration found"

echo ""
echo "📋 Next Steps for Railway Deployment:"
echo "====================================='"
echo ""
echo "1. Create a Railway account at https://railway.app"
echo ""
echo "2. Install Railway CLI:"
echo "   • Option A: npm install -g @railway/cli"
echo "   • Option B: Manual download from GitHub"
echo ""
echo "3. Login to Railway:"
echo "   railway login"
echo ""
echo "4. Initialize project:"
echo "   railway init"
echo ""
echo "5. Add PostgreSQL database:"
echo "   railway add postgresql"
echo ""
echo "6. Set environment variables:"
echo "   railway variables set AUTO_UPDATE_SECRET_KEY=your_secret_here"
echo "   railway variables set HOST=0.0.0.0"
echo ""
echo "7. Deploy:"
echo "   railway up"
echo ""
echo "🔗 Alternative: GitHub Integration"
echo "=================================="
echo "1. Push your code to GitHub"
echo "2. Connect GitHub repo to Railway dashboard"
echo "3. Set environment variables in Railway UI"
echo "4. Deploy automatically"
echo ""
echo "🐳 Docker Build Notes:"
echo "====================="
echo "• Fixed musl target compilation issues"
echo "• Uses standard GNU libc build (more reliable)"
echo "• Includes dependency caching for faster builds"
echo "• Non-root user for security"
echo ""
echo "📁 Files created for deployment:"
echo "• Dockerfile - Fixed container configuration"
echo "• Dockerfile.secure - Alternative with health checks"
echo "• railway.json - Railway deployment config"
echo "• .dockerignore - Docker build optimization"
echo "• RAILWAY_DEPLOY.md - Detailed deployment guide"
echo ""
echo "✅ Ready for Railway deployment!"

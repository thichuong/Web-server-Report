#!/bin/bash

# Railway Memory-Optimized Build Script
echo "🔧 Railway Memory-Optimized Deployment"
echo "======================================"

# Test local build first
echo "Testing local build..."
if cargo build --release; then
    echo "✅ Local build successful"
else
    echo "❌ Local build failed"
    exit 1
fi

echo ""
echo "📊 Available Dockerfiles:"
echo "========================"
echo "1. Dockerfile (distroless - smallest runtime)"
echo "2. Dockerfile.alpine (alpine-based - ultra lightweight)"
echo "3. Dockerfile.secure (debian-based - most compatible)"

echo ""
echo "🚀 Railway Deployment Options:"
echo "=============================="

echo ""
echo "Option 1: Use distroless (recommended for Railway)"
echo "railway.json already configured for main Dockerfile"
echo "Commands:"
echo "  railway login"
echo "  railway init"
echo "  railway add postgresql"
echo "  railway variables set AUTO_UPDATE_SECRET_KEY=your_secret"
echo "  railway variables set HOST=0.0.0.0"
echo "  railway up"

echo ""
echo "Option 2: Use Alpine (if distroless fails)"
echo "Commands:"
echo "  # First, update railway.json to use Dockerfile.alpine"
echo "  railway login"
echo "  railway init"
echo "  railway add postgresql"
echo "  railway variables set AUTO_UPDATE_SECRET_KEY=your_secret"
echo "  railway variables set HOST=0.0.0.0"
echo "  railway up"

echo ""
echo "🧠 Memory Optimization Tips:"
echo "==========================="
echo "• Distroless runtime uses minimal memory"
echo "• Alpine version is even smaller but may have compatibility issues"
echo "• Build happens in isolated environment with sufficient resources"
echo "• Runtime memory usage is very low"

echo ""
echo "🩺 Troubleshooting Exit Code 137:"
echo "================================"
echo "• This usually means the build process was killed (out of memory)"
echo "• Railway should have sufficient build resources"
echo "• The new Dockerfile uses distroless runtime to minimize memory"
echo "• If issues persist, try the Alpine version"

echo ""
echo "🔄 Quick Switch to Alpine:"
echo "========================="
read -p "Switch to Alpine Dockerfile? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cp railway.json railway.json.bak
    sed 's/"dockerfilePath": "Dockerfile"/"dockerfilePath": "Dockerfile.alpine"/' railway.json.bak > railway.json
    echo "✅ Switched to Alpine Dockerfile"
    echo "ℹ️  You can revert with: mv railway.json.bak railway.json"
fi

echo ""
echo "✅ Ready for Railway deployment with memory optimizations!"

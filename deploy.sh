#!/bin/bash

# Deploy script for Railway

echo "🚀 Deploying to Railway..."

# Check if git repo is initialized
if [ ! -d .git ]; then
    echo "📝 Initializing git repository..."
    git init
fi

# Add all files
echo "📦 Adding files to git..."
git add .

# Commit changes
echo "💾 Committing changes..."
git commit -m "Deploy to Railway: $(date)"

# Check if railway CLI is installed
if ! command -v railway &> /dev/null; then
    echo "❌ Railway CLI not found. Please install it first:"
    echo "npm install -g @railway/cli"
    echo ""
    echo "Or deploy via GitHub:"
    echo "1. Push this repo to GitHub"
    echo "2. Connect GitHub repo to Railway"
    echo "3. Add environment variables in Railway dashboard"
    exit 1
fi

# Deploy via Railway CLI
echo "🚂 Deploying via Railway CLI..."
railway up

echo "✅ Deployment complete!"
echo "🌐 Check your Railway dashboard for the live URL"

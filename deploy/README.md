# Deploy Directory

This directory contains all deployment configurations and Docker files for the web server.

## 📁 Contents

### Docker Files
- `Dockerfile.fixed` - **RECOMMENDED** - Working solution with base64ct dependency pinning
- `Dockerfile` - Original Dockerfile  
- `Dockerfile.optimized` - Optimized multi-stage build
- `Dockerfile.railway` - Railway-specific (has edition2024 issues)
- `Dockerfile.simple` - Simplified build
- `Dockerfile.ubuntu` - Ubuntu-based build
- `Dockerfile.nightly` - Rust nightly (experimental)
- `Dockerfile.pinned` - Alternative pinning approach
- `Dockerfile.unlocked` - Build without lock file

### Platform Configurations
- `railway.json` - Railway deployment configuration (uses Dockerfile.fixed)
- `nixpacks.toml` - Nixpacks configuration
- `Procfile` - Process configuration

### Documentation
- `DOCKER_BUILD_RESOLUTION.md` - Detailed solution for edition2024 issues
- `RAILWAY_DEPLOYMENT.md` - Railway deployment guide

### Scripts
- `deploy_railway.sh` - Railway deployment script (if exists)

## 🚀 Deployment Commands

### Local Docker Build
```bash
# Recommended - working build
docker build -t web-server-report -f deploy/Dockerfile.fixed .

# Test the container
docker run -p 8000:8000 web-server-report
```

### Railway Deployment
```bash
# Deploy to Railway (uses railway.json config)
railway up
```

## 🎯 Current Status
- ✅ Docker build working with `Dockerfile.fixed`
- ✅ Railway configuration ready
- ✅ Edition2024 dependency conflicts resolved
- ✅ Multi-stage optimized build

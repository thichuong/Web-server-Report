# Docker Build Troubleshooting Guide

## ✅ Issue Fixed: Musl Target Compilation Error

**Error:**
```
process "/bin/bash -ol pipefail -c cargo build --release --target x86_64-unknown-linux-musl" did not complete successfully: exit code: 101
```

**Root Cause:**
The original Dockerfile was trying to cross-compile to `x86_64-unknown-linux-musl` target without proper musl toolchain setup.

**Solution Applied:**
1. **Removed musl target** - Use standard GNU libc compilation
2. **Added SSL dependencies** - `pkg-config` and `libssl-dev` for sqlx-postgres
3. **Optimized build process** - Separate dependency and source builds
4. **Enhanced security** - Non-root user, proper permissions

## 🔧 Build Process Improvements

### Before (Problematic):
```dockerfile
FROM rust:1.75 as builder
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl
```

### After (Fixed):
```dockerfile
FROM rust:1.75-bookworm as builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src
COPY src ./src
RUN cargo build --release
```

## 🐳 Alternative Dockerfiles Available

### 1. `Dockerfile` (Main)
- Standard build, reliable for Railway
- Uses Rust 1.75-bookworm
- GNU libc target (most compatible)

### 2. `Dockerfile.secure` (Enhanced)
- Includes health checks
- More recent base images
- Additional security features

## 🚀 Deployment Commands

**Test locally:**
```bash
./deploy.sh  # Includes build verification
```

**Deploy to Railway:**
```bash
railway login
railway init
railway add postgresql
railway variables set AUTO_UPDATE_SECRET_KEY=your_secret
railway variables set HOST=0.0.0.0
railway up
```

## 🔍 Common Issues & Solutions

### Issue: SSL/TLS Dependencies
**Error:** `failed to run custom build command for openssl-sys`
**Solution:** ✅ Already fixed - added `pkg-config libssl-dev`

### Issue: Permission Denied
**Error:** `permission denied while trying to connect to the Docker daemon`
**Solution:** Ensure Docker service is running and user has permissions

### Issue: Out of Memory
**Error:** `signal: killed` during build
**Solution:** Railway provides sufficient memory, but ensure code doesn't have memory leaks

### Issue: Static Files Not Found
**Error:** `static/home.html` not found
**Solution:** ✅ Already fixed - properly copied in Dockerfile

## ✅ Build Verification

The deployment script now includes automatic build verification:
```bash
if cargo build --release > /dev/null 2>&1; then
    echo "✅ Local build successful"
else
    echo "❌ Local build failed"
    exit 1
fi
```

Your Rust application is now ready for reliable Railway deployment! 🚀

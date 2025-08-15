# Docker Build Troubleshooting Guide

## 🚨 Latest Issue Fixed: Memory Error (Exit Code 137)

**Error:**
```
process "/bin/sh -c apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*" 
did not complete successfully: exit code: 137: context canceled
```

**Root Cause:**
- Exit code 137 = SIGKILL (process killed due to memory constraints)
- Railway build environment has memory limits
- Large Docker images can exceed available memory during build

**Solution Applied:**
1. **Switched to distroless runtime** - Minimal memory footprint
2. **Optimized build layers** - Reduced intermediate image sizes
3. **Eliminated apt-get in runtime** - No package manager overhead
4. **Added Alpine alternative** - Ultra-lightweight option

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

## 🐳 Available Dockerfiles (Memory Optimized)

### 1. `Dockerfile` (Main - Distroless)
- **Memory optimized** - Uses Google's distroless base
- Minimal runtime footprint (~20MB)
- No package manager or shell in runtime
- Best for production deployment

### 2. `Dockerfile.alpine` (Ultra-lightweight)
- Uses Alpine Linux (~5MB base)
- Smallest possible image size
- May have glibc compatibility issues
- Fallback option if distroless fails

### 3. `Dockerfile.secure` (Compatible)
- Uses Debian slim base
- Includes health checks
- Larger but most compatible
- Use if others fail

## 🚀 Deployment Commands (Memory Optimized)

**Test locally:**
```bash
./deploy-optimized.sh  # Interactive memory optimization
```

**Quick Railway Deploy:**
```bash
railway login
railway init
railway add postgresql
railway variables set AUTO_UPDATE_SECRET_KEY=your_secret
railway variables set HOST=0.0.0.0
railway up
```

**Switch to Alpine if needed:**
```bash
# Update railway.json to use Dockerfile.alpine
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

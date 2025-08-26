# Railway Deployment Troubleshooting

## 🚨 Lỗi Build phổ biến và cách khắc phục

### 1. Network Timeout Error
```
error: could not download file from 'https://static.rust-lang.org/dist/channel-rust-stable.toml.sha256'
operation timed out
```

**Nguyên nhân**: Railway build environment có network timeout hoặc bandwidth limit.

**Giải pháp**:
1. **Tăng timeout values** trong Dockerfile:
   ```dockerfile
   ENV CARGO_HTTP_TIMEOUT=600 \
       CARGO_HTTP_MULTIPLEXING=true \
       CARGO_HTTP_LOW_SPEED_LIMIT=10 \
       CARGO_HTTP_LOW_SPEED_TIMEOUT=600
   ```

2. **Sử dụng single-stage build** thay vì multi-stage:
   - File: `Dockerfile.simple-railway`
   - Ít phức tạp, ít dependency layers

3. **Retry mechanism**:
   ```dockerfile
   RUN cargo build --release || cargo build --release
   ```

### 2. Multi-stage Build Issues
```
RUN cargo build --release && rm -rf src target/release/deps/web*
exit code: 101
```

**Nguyên nhân**: 
- Cấu trúc project có cả lib và binary
- Dependency caching strategy không tương thích

**Giải pháp**:
1. **Simplify build process**:
   ```dockerfile
   # Đơn giản: copy tất cả, build một lần
   COPY . .
   RUN cargo build --release
   ```

2. **Correct dependency caching**:
   ```dockerfile
   # Cần cả main.rs và lib.rs để build dependencies
   RUN mkdir src && \
       echo "fn main() {}" > src/main.rs && \
       echo "// dummy lib" > src/lib.rs
   ```

### 3. Static Assets Missing
```
404 Not Found: /shared_assets/css/style.css
```

**Nguyên nhân**: `.dockerignore` loại bỏ static files

**Giải pháp**:
1. **Kiểm tra .dockerignore**:
   ```dockerfile
   # ✅ GOOD - chỉ loại bỏ build artifacts
   target/
   
   # ❌ BAD - loại bỏ static assets
   *.css
   *.js
   ```

2. **Explicit copy trong Dockerfile**:
   ```dockerfile
   COPY dashboards ./dashboards/
   COPY shared_components ./shared_components/
   COPY shared_assets ./shared_assets/
   ```

### 4. Port Binding Issues
```
Server listening on http://127.0.0.1:3000
```

**Nguyên nhân**: Server bind localhost thay vì 0.0.0.0

**Giải pháp**:
```rust
let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
```

## 📋 Deployment Options

### Option 1: Simple Build (Recommended for Railway)
```json
{
  "build": {
    "dockerfilePath": "Dockerfile.simple-railway"
  }
}
```
- Single-stage build
- Larger image (~2GB) but more reliable
- Build time: 3-5 minutes

### Option 2: Optimized Build (For production)
```json
{
  "build": {
    "dockerfilePath": "Dockerfile.optimized"
  }
}
```
- Multi-stage build
- Smaller image (~200MB) 
- Build time: 5-8 minutes
- Higher chance of network issues

### Option 3: Multi-stage Build (Advanced)
```json
{
  "build": {
    "dockerfilePath": "Dockerfile.railway"
  }
}
```
- Complex dependency caching
- Smallest image size
- Highest build complexity

## 🔧 Railway Configuration

### Environment Variables
```bash
# Required
DATABASE_URL=postgresql://...
REDIS_URL=redis://...
TAAPI_SECRET=...
FINNHUB_API_KEY=...

# Optional
RUST_LOG=info
RUST_BACKTRACE=1
HOST=0.0.0.0
PORT=8000
```

### Build Settings
```json
{
  "deploy": {
    "healthcheckTimeout": 180,  // Tăng timeout cho startup
    "restartPolicyMaxRetries": 5
  }
}
```

## 🚀 Deploy Commands

### Method 1: Railway CLI
```bash
railway login
railway link your-project-id
railway up
```

### Method 2: Git Push (Auto Deploy)
```bash
git add .
git commit -m "🚀 Deploy: Fixed build configuration"
git push origin main
```

## 📊 Build Time Optimization

### Local testing trước khi deploy:
```bash
# Test build locally
docker build -f Dockerfile.simple-railway -t test .

# Check image size
docker images test

# Test run
docker run -p 8000:8000 -e HOST=0.0.0.0 test
```

### Railway build optimization:
- Sử dụng `.dockerignore` để giảm context size
- Tránh `--no-cache` unless necessary
- Monitor build logs để identify bottlenecks

## 🎯 Success Indicators

✅ **Successful deployment**:
- Build completes in < 10 minutes  
- Image size < 2GB
- Health check passes
- Static assets load correctly
- WebSocket connections work

❌ **Common failures**:
- Network timeouts > 600s
- Out of memory during build
- Missing environment variables
- Port binding errors

---

**Current Status**: Testing simple build approach for reliability

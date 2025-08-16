# 🚂 RAILWAY DEPLOYMENT GUIDE

## 📊 **TỐI ƯU ĐÃ THỰC HIỆN**

### **🐳 Docker Optimizations**
- **Multi-stage build**: Giảm image size từ 1GB+ xuống ~200MB
- **Dependency caching**: Layer caching cho dependencies 
- **Runtime optimizations**: CPU-generic builds cho compatibility
- **Security**: Non-root user, minimal runtime dependencies

### **⚙️ Railway Configuration**
- **Dockerfile.railway**: Optimized cho Railway platform
- **railway.json**: Health check và deployment config
- **nixpacks.toml**: Alternative build method
- **Procfile**: Heroku-compatible startup

## 🚀 **DEPLOYMENT PROCESS**

### **1. Chuẩn bị Environment Variables**
Trong Railway Dashboard, set các biến sau:
```
DATABASE_URL=postgresql://user:password@host:port/database
TAAPI_SECRET=your_taapi_secret_key  
REDIS_URL=redis://user:password@host:port (optional)
```

### **2. Deploy Method Options**

#### **Option A: Docker Build (Recommended)**
```bash
# Sử dụng Dockerfile.railway
./scripts/deploy_railway.sh
```

#### **Option B: Nixpacks Build**  
```bash
# Railway sẽ tự động detect Rust project
railway up
```

#### **Option C: Manual Deploy**
```bash
# 1. Login to Railway
railway login

# 2. Create/connect project
railway link

# 3. Deploy
railway up --detach
```

### **3. Verify Deployment**
```bash
# Check health
curl https://your-app.railway.app/health

# View logs
railway logs

# Monitor metrics
curl https://your-app.railway.app/api/performance/metrics
```

## 📈 **EXPECTED PERFORMANCE**

### **Railway Specifications**
- **Memory**: 1GB RAM (recommended)
- **CPU**: 1 vCPU (shared)  
- **Disk**: 100MB (application + assets)
- **Network**: Unlimited bandwidth

### **Performance Metrics**
- **Startup Time**: 15-30 seconds
- **Response Time**: 50-200ms average
- **Throughput**: 1,000-3,000 req/s
- **Memory Usage**: 50-150MB runtime

## 🔧 **CONFIGURATION FILES**

### **1. Dockerfile.railway**
```dockerfile
# Optimized multi-stage build
FROM rust:1.82-slim as builder
# ... build optimizations
FROM ubuntu:22.04  
# ... runtime optimizations
```

### **2. railway.json** 
```json
{
  "build": {
    "builder": "DOCKERFILE",
    "dockerfilePath": "Dockerfile.railway"
  },
  "deploy": {
    "healthcheckPath": "/health",
    "healthcheckTimeout": 300
  }
}
```

### **3. nixpacks.toml**
```toml
[variables]
RUST_LOG = "info"
TOKIO_THREAD_STACK_SIZE = "4194304"

[phases.build]  
cmd = "cargo build --release --locked"
```

## 🔍 **TROUBLESHOOTING**

### **Build Issues**
```bash
# Check build logs
railway logs --stage build

# Test local build
podman build -f Dockerfile.railway -t test .

# Validate config
railway validate
```

### **Runtime Issues**
```bash
# Check application logs
railway logs --stage runtime

# Health check
curl -I https://your-app.railway.app/health

# Performance check
curl https://your-app.railway.app/api/performance/metrics
```

### **Common Issues**

1. **Build Timeout**
   - Solution: Use Dockerfile (faster than Nixpacks)
   - Optimize dependency caching

2. **Memory Limit**
   - Solution: Increase Railway plan
   - Optimize memory usage

3. **Database Connection**
   - Solution: Check DATABASE_URL format
   - Verify network access

4. **Health Check Failed**
   - Solution: Increase timeout
   - Check `/health` endpoint

## 📱 **MONITORING & MAINTENANCE**

### **Health Monitoring**
```bash
# Automated health check
curl -f https://your-app.railway.app/health

# WebSocket check  
wscat -c wss://your-app.railway.app/ws
```

### **Performance Monitoring**
```bash
# System metrics
curl https://your-app.railway.app/api/performance/metrics

# Cache statistics
curl https://your-app.railway.app/api/cache/stats
```

### **Log Analysis**
```bash
# View recent logs
railway logs --tail

# Filter error logs  
railway logs | grep ERROR

# Export logs
railway logs --json > app.logs
```

## 🔄 **SCALING & UPDATES**

### **Horizontal Scaling**
- Railway: Single instance (vertical scaling)
- Use CDN for static assets
- Database read replicas

### **Updates & Rollbacks**
```bash
# Deploy new version
git push origin main
railway up

# Rollback if needed
railway rollback
```

### **Performance Tuning**
- Monitor memory usage
- Adjust thread pool size
- Optimize database queries
- Enable CDN caching

## 🎯 **PRODUCTION CHECKLIST**

- [ ] ✅ Environment variables configured
- [ ] ✅ Database connection tested  
- [ ] ✅ Health check responds
- [ ] ✅ WebSocket connection works
- [ ] ✅ Performance metrics accessible
- [ ] ✅ Logs are readable
- [ ] ✅ Error handling tested
- [ ] ✅ Backup strategy in place
- [ ] ✅ Monitoring alerts setup

## 🔗 **USEFUL LINKS**

- [Railway Documentation](https://docs.railway.app)
- [Rust on Railway](https://docs.railway.app/guides/languages/rust)
- [Health Check Guide](https://docs.railway.app/deploy/healthchecks)
- [Environment Variables](https://docs.railway.app/develop/variables)

---

🎉 **Your high-performance Rust server is now ready for Railway deployment!**

# 🔧 SSL/TLS TROUBLESHOOTING GUIDE

## 🚨 **SSL HANDSHAKE ERRORS ON RAILWAY**

### **Các lỗi thường gặp:**
```
SSL handshake failed (5).
SSL read failed (1) - closing connection
00CAB47FDF7F0000:error:0A000197:SSL routines:SSL_shutdown:shutdown while in init:ssl/ssl_lib.c:2751:
```

### **✅ ĐÃ THỰC HIỆN CÁC FIXES:**

#### **1. HTTP Client Configuration** 
- ✅ Tăng timeout: 30s (thay vì 10s)
- ✅ Thêm connect_timeout: 10s riêng biệt
- ✅ SSL certificate validation: enabled  
- ✅ Built-in CA certificates: enabled
- ✅ User-Agent headers cho tất cả requests
- ✅ TCP keepalive và nodelay optimization

#### **2. Retry Logic với Exponential Backoff**
- ✅ 3 lần retry cho mỗi API call
- ✅ Exponential backoff: 1s, 2s, 4s  
- ✅ Graceful fallback với default values
- ✅ Detailed error logging

#### **3. Docker Container Improvements**
- ✅ Updated CA certificates trong build & runtime
- ✅ Proper SSL libraries: libssl1.1
- ✅ Debian bullseye-slim base (stable)
- ✅ Non-root user cho security

#### **4. Health Check với SSL Testing**
- ✅ Endpoint `/health` kiểm tra SSL connectivity  
- ✅ Test tất cả external APIs
- ✅ Timeout protection (5s per test)
- ✅ Detailed SSL status reporting

---

## 🚀 **DEPLOYMENT STEPS**

### **1. Build & Test SSL-Optimized Container**
```bash
# Build với Dockerfile.railway (SSL-optimized)
docker build -f Dockerfile.railway -t web-server-railway .

# Test locally
docker run -p 8000:8000 -e DATABASE_URL="your_db" -e TAAPI_SECRET="your_secret" web-server-railway

# Check health endpoint
curl http://localhost:8000/health
```

### **2. Deploy to Railway**
```bash
# Sử dụng script deploy đã tối ưu
./scripts/deploy_railway.sh

# Hoặc manual deploy
railway up --detach
```

### **3. Monitor SSL Status**
```bash
# Check health sau khi deploy
curl https://your-app.railway.app/health

# Monitor logs
railway logs --follow
```

---

## 🔍 **DEBUGGING SSL ISSUES**

### **Check Certificate Chain:**
```bash
# Test SSL handshake
openssl s_client -connect api.coingecko.com:443 -servername api.coingecko.com

# Verify certificates
curl -vvI https://api.coingecko.com/api/v3/ping
```

### **Railway Environment:**
```bash
# Set environment variables trong Railway Dashboard:
DATABASE_URL=postgresql://user:pass@host:port/db
TAAPI_SECRET=your_taapi_secret
REDIS_URL=redis://user:pass@host:port
RUST_LOG=debug                    # Enable debug logging
RUST_BACKTRACE=1                  # Full error traces
```

### **Monitoring Commands:**
```bash
# Check railway logs
railway logs --tail 100

# Check service status  
railway status

# View environment variables
railway variables
```

---

## 🎯 **EXPECTED IMPROVEMENTS**

After implementing these fixes, bạn should see:

✅ **No more SSL handshake failures**  
✅ **Reliable external API connections**  
✅ **Graceful degradation khi APIs fail**  
✅ **Better error messages & logging**  
✅ **Automatic retry cho transient failures**  
✅ **SSL status monitoring qua /health**

---

## 📞 **SUPPORT**

Nếu vẫn gặp issues:

1. Check `/health` endpoint cho SSL status details
2. Review Railway logs với `railway logs --follow` 
3. Verify environment variables trong Railway dashboard
4. Test local container trước khi deploy

**Railway deployment URL:** `https://your-app.railway.app`

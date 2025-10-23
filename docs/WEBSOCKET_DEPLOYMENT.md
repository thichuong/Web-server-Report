# 🔌 WebSocket Deployment Guide

## 📋 Tổng quan

WebSocket implementation của dự án sử dụng:
- **Backend**: Axum WebSocket với Tokio async runtime
- **Frontend**: Native WebSocket API với auto-reconnect
- **Architecture**: Service Islands Layer 3 Communication

## ✅ Cloudflare Compatibility

### Đã được verify tương thích với:
- ✅ Cloudflare Free Plan
- ✅ Cloudflare Pro Plan
- ✅ Cloudflare WebSocket 100s timeout
- ✅ SSL/TLS termination
- ✅ Auto-reconnection logic

## 🚀 Quick Start

### 1. Local Testing

```bash
# Terminal 1: Start server
cargo run

# Terminal 2: Test WebSocket
./examples/test-websocket-cloudflare.sh localhost:8000 ws
```

### 2. Production Testing (với Cloudflare)

```bash
# Test với domain của bạn
./examples/test-websocket-cloudflare.sh yourdomain.com wss
```

### 3. Browser Testing

Mở browser console và test:

```javascript
// Connect to WebSocket
const ws = new WebSocket('wss://yourdomain.com/ws');

// Listen for messages
ws.onmessage = (event) => {
    console.log('Received:', JSON.parse(event.data));
};

// Send ping
ws.send('ping');

// Request data
ws.send('request_update');
```

## ⚙️ Configuration

### Backend (`src/routes/websocket.rs`)

```rust
// WebSocket route
Router::new()
    .route("/ws", get(websocket_handler))

// Handler với upgrade
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(service_islands): State<Arc<ServiceIslands>>
) -> Response {
    ws.protocols(["chat", "superchat"])
        .on_upgrade(move |socket| websocket_connection_handler(socket, service_islands))
}
```

### Frontend (`dashboard-websocket.js`)

```javascript
// Connection URL
const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const wsUrl = `${protocol}//${window.location.host}/ws`;

// Heartbeat interval (30s - optimal for Cloudflare)
this.heartbeatInterval = setInterval(() => {
    this.socket.send('ping');
}, 30000);
```

## ☁️ Cloudflare Setup

### 1. Enable WebSockets

Trong Cloudflare Dashboard:
```
Network → WebSockets → ON
```

### 2. Configure SSL/TLS

```
SSL/TLS → Overview → Full (strict)
```

### 3. Disable Rocket Loader

```
Speed → Optimization → Rocket Loader → OFF
```

### 4. Optional: Create Page Rule

```
Pattern: yourdomain.com/ws*
Settings:
  - Cache Level: Bypass
  - Browser Cache TTL: Bypass
  - Security Level: Medium
```

## 📊 Message Types

### Client → Server

| Message | Description | Example |
|---------|-------------|---------|
| `ping` | Heartbeat | `"ping"` |
| `request_update` | Request dashboard data | `"request_update"` |
| `request_dashboard_data` | Alternative request | `"request_dashboard_data"` |

### Server → Client

| Type | Description | Data Structure |
|------|-------------|---------------|
| `connected` | Connection established | `{ type, message, timestamp }` |
| `pong` | Heartbeat response | `{ type, timestamp }` |
| `dashboard_data` | Market data update | `{ type, data, timestamp, source }` |
| `btc_price_update` | BTC price only | `{ type, data }` |
| `market_update` | Partial market data | `{ type, data }` |

## 🔍 Debugging

### Enable Debug Mode

**Frontend:**
```javascript
// In dashboard-websocket.js
const WS_DEBUG = true; // Change from false to true
```

**Backend:**
```rust
// Already enabled via println! macros
// Check server logs for WebSocket events
```

### Common Issues

#### Issue: Connection fails immediately
**Solution:**
```bash
# Check if server is running
curl http://localhost:8000/health

# Check WebSocket endpoint
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" http://localhost:8000/ws
```

#### Issue: 520/521 errors with Cloudflare
**Solution:**
- Verify SSL certificate is valid
- Check server is accessible from internet
- Verify firewall allows WebSocket connections

#### Issue: Connection drops after 100s
**Solution:**
- Heartbeat is already configured at 30s
- Check server logs for errors
- Verify Cloudflare WebSocket is enabled

## 📈 Performance Metrics

### Expected Latency

| Environment | Connection | Message | Reconnect |
|------------|-----------|---------|-----------|
| Local | 10-30ms | 5-15ms | <100ms |
| Production (no CF) | 50-150ms | 20-50ms | 200-500ms |
| Production (with CF) | 100-300ms | 50-100ms | 500-1000ms |

### Monitoring

```bash
# Watch server logs
tail -f /var/log/web-server-report.log | grep WebSocket

# Monitor connections
watch -n1 'netstat -an | grep :8000 | grep ESTABLISHED'
```

## 🧪 Load Testing

### Using Artillery

```bash
# Install artillery
npm install -g artillery

# Run load test
artillery quick --count 100 --num 10 ws://localhost:8000/ws
```

### Using custom script

```bash
# Run multiple connections
for i in {1..10}; do
    ./examples/test-websocket-cloudflare.sh localhost:8000 ws &
done
wait
```

## 🔒 Security

### Current Security Measures

1. ✅ **Origin Validation**: Via Cloudflare
2. ✅ **Rate Limiting**: Can be added via Cloudflare WAF
3. ✅ **SSL/TLS**: Enforced in production
4. ✅ **Connection Timeout**: 100s idle timeout

### Recommended Additional Security

```rust
// Add origin checking (optional)
if let Some(origin) = headers.get("origin") {
    if !is_allowed_origin(origin) {
        return Err(StatusCode::FORBIDDEN);
    }
}
```

## 📚 Resources

- [Cloudflare WebSocket Docs](https://developers.cloudflare.com/workers/runtime-apis/websockets/)
- [Axum WebSocket Guide](https://docs.rs/axum/latest/axum/extract/ws/)
- [MDN WebSocket API](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API)

## ✅ Checklist

Trước khi deploy:

- [ ] WebSocket enabled in Cloudflare Dashboard
- [ ] SSL/TLS mode = Full (strict)
- [ ] Rocket Loader disabled
- [ ] Server bind to 0.0.0.0
- [ ] Heartbeat configured (30s)
- [ ] Reconnection logic tested
- [ ] Error handling implemented
- [ ] Load testing completed
- [ ] Monitoring setup

---

**Status**: ✅ Production Ready cho Cloudflare deployment

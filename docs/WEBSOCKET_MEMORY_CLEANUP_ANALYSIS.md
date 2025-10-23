# 🔍 WebSocket Memory Cleanup Analysis

## 📊 Current Implementation Analysis

### ✅ **Memory Cleanup - GỐC RỄ**

#### **1. Automatic Cleanup khi connection đóng**

**File: `src/routes/websocket.rs`**

```rust
async fn websocket_connection_handler(
    mut socket: axum::extract::ws::WebSocket,  // ❌ MUT - owned by handler
    service_islands: Arc<ServiceIslands>        // ✅ Arc - shared, reference counted
) {
    // ...
    
    let mut broadcast_rx = service_islands.websocket_service
        .get_broadcast_tx()
        .subscribe();  // ❌ VẤN ĐỀ: Receiver không được cleanup rõ ràng
    
    loop {
        tokio::select! {
            broadcast_result = broadcast_rx.recv() => { ... }
            client_message = socket.recv() => {
                match client_message {
                    Some(Ok(axum::extract::ws::Message::Close(_))) => {
                        println!("🔌 WebSocket client disconnected");
                        break;  // ✅ Exit loop
                    }
                    Some(Err(e)) => {
                        println!("❌ WebSocket error: {}", e);
                        break;  // ✅ Exit loop
                    }
                    None => {
                        println!("🔌 WebSocket stream ended");
                        break;  // ✅ Exit loop
                    }
                    _ => {}
                }
            }
        }
    }
    
    println!("🔌 WebSocket connection handler finished");
    // ❌ THIẾU: Explicit cleanup code
}
```

### 🔴 **Vấn đề phát hiện:**

#### **Problem 1: Broadcast Receiver không được drop rõ ràng**

**Hiện trạng:**
```rust
let mut broadcast_rx = service_islands.websocket_service
    .get_broadcast_tx()
    .subscribe();
```

**Vấn đề:**
- `broadcast_rx` là `tokio::sync::broadcast::Receiver`
- Khi function kết thúc, Rust sẽ tự động drop
- NHƯNG: Không có log hoặc explicit cleanup
- RISK: Nếu có lỗi trong drop, không biết được

#### **Problem 2: Socket không được close rõ ràng**

**Hiện trạng:**
```rust
async fn websocket_connection_handler(
    mut socket: axum::extract::ws::WebSocket,
    // ...
) {
    // ... loop ...
    println!("🔌 WebSocket connection handler finished");
    // Socket tự động drop ở đây
}
```

**Vấn đề:**
- Socket không được `.close()` explicitly
- Dựa vào Rust's RAII để cleanup
- RISK: Nếu client crash, server có thể giữ connection lâu

#### **Problem 3: Không track active connections**

**Hiện trạng:**
- Không có counter hoặc tracking mechanism
- Không biết có bao nhiêu connections đang active
- Không có way để force cleanup all connections

### ❌ **Memory Leaks Potential:**

1. **Broadcast Channel:**
   - Buffer size: 1000 messages
   - Nếu slow consumer → messages bị drop
   - Nhưng memory vẫn allocated cho buffer

2. **Zombie Connections:**
   - Nếu client crash mà không send Close frame
   - Server sẽ đợi đến khi timeout
   - THIẾU: Connection timeout mechanism

3. **Task Leaks:**
   - Mỗi connection = 1 tokio task
   - Nếu task panic mà không cleanup → leak

## 🔧 **Recommended Fixes**

### **Fix 1: Explicit Cleanup với Drop Guard**

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Connection guard để track và cleanup
struct ConnectionGuard {
    id: usize,
    active_connections: Arc<AtomicUsize>,
}

impl ConnectionGuard {
    fn new(active_connections: Arc<AtomicUsize>) -> Self {
        let id = active_connections.fetch_add(1, Ordering::SeqCst);
        println!("✅ WebSocket connection #{} established. Total: {}", 
                 id + 1, active_connections.load(Ordering::SeqCst));
        Self { id, active_connections }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let remaining = self.active_connections.fetch_sub(1, Ordering::SeqCst) - 1;
        println!("🧹 Cleaning up WebSocket connection #{}. Remaining: {}", 
                 self.id, remaining);
    }
}

async fn websocket_connection_handler(
    mut socket: axum::extract::ws::WebSocket,
    service_islands: Arc<ServiceIslands>
) {
    // Create guard - auto cleanup when dropped
    let _guard = ConnectionGuard::new(service_islands.active_ws_connections.clone());
    
    // ... rest of handler ...
}
```

### **Fix 2: Explicit Socket Close**

```rust
async fn websocket_connection_handler(
    mut socket: axum::extract::ws::WebSocket,
    service_islands: Arc<ServiceIslands>
) {
    // ... existing code ...
    
    loop {
        tokio::select! {
            // ... existing select branches ...
        }
    }
    
    // Explicit cleanup TRƯỚC KHI function kết thúc
    println!("🧹 Cleaning up WebSocket connection...");
    
    // 1. Close socket gracefully
    if let Err(e) = socket.close().await {
        println!("⚠️ Error closing socket: {}", e);
    }
    
    // 2. broadcast_rx sẽ tự động drop ở đây
    println!("✅ WebSocket connection fully cleaned up");
}
```

### **Fix 3: Connection Timeout**

```rust
use tokio::time::{timeout, Duration};

async fn websocket_connection_handler(
    mut socket: axum::extract::ws::WebSocket,
    service_islands: Arc<ServiceIslands>
) {
    let _guard = ConnectionGuard::new(service_islands.active_ws_connections.clone());
    let mut broadcast_rx = service_islands.websocket_service.get_broadcast_tx().subscribe();
    
    // Add connection timeout
    let connection_timeout = Duration::from_secs(300); // 5 minutes idle
    let mut last_activity = tokio::time::Instant::now();
    
    loop {
        // Check timeout
        if last_activity.elapsed() > connection_timeout {
            println!("⏰ Connection timeout - closing due to inactivity");
            break;
        }
        
        tokio::select! {
            broadcast_result = broadcast_rx.recv() => {
                match broadcast_result {
                    Ok(broadcast_message) => {
                        last_activity = tokio::time::Instant::now(); // Reset timeout
                        // ... send message ...
                    }
                    Err(e) => {
                        println!("⚠️ Broadcast channel closed: {}", e);
                        break;
                    }
                }
            }
            
            client_message = socket.recv() => {
                last_activity = tokio::time::Instant::now(); // Reset timeout
                // ... handle message ...
            }
            
            // Timeout check branch
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                // Periodic check - continue loop
            }
        }
    }
    
    // Cleanup
    println!("🧹 Cleaning up WebSocket connection...");
    let _ = socket.close().await;
    println!("✅ WebSocket connection fully cleaned up");
}
```

### **Fix 4: Active Connections Tracking**

**Add to `ServiceIslands`:**

```rust
pub struct ServiceIslands {
    // ... existing fields ...
    
    /// Track active WebSocket connections
    pub active_ws_connections: Arc<AtomicUsize>,
}

impl ServiceIslands {
    pub async fn initialize() -> Result<Self, anyhow::Error> {
        // ... existing init code ...
        
        Ok(Self {
            // ... existing fields ...
            active_ws_connections: Arc::new(AtomicUsize::new(0)),
        })
    }
    
    /// Get number of active WebSocket connections
    pub fn active_connections(&self) -> usize {
        self.active_ws_connections.load(Ordering::SeqCst)
    }
}
```

**Add health check endpoint:**

```rust
// In routes/system.rs
async fn websocket_stats(
    State(service_islands): State<Arc<ServiceIslands>>
) -> Json<serde_json::Value> {
    Json(json!({
        "active_connections": service_islands.active_connections(),
        "status": "healthy"
    }))
}
```

## 📊 **Testing Memory Cleanup**

### **Test 1: Normal Disconnect**

```bash
# Connect
wscat -c ws://localhost:8000/ws

# Type: exit
# Check server logs for cleanup message
```

### **Test 2: Force Kill Client**

```bash
# Connect in background
wscat -c ws://localhost:8000/ws &
PID=$!

# Kill immediately
kill -9 $PID

# Check server logs - should cleanup within timeout
```

### **Test 3: Memory Leak Test**

```bash
# Create 100 connections and kill them
for i in {1..100}; do
    wscat -c ws://localhost:8000/ws &
    PID=$!
    sleep 0.1
    kill -9 $PID
done

# Check memory usage
ps aux | grep web-server-report
```

### **Test 4: Connection Tracking**

```bash
# Start server
cargo run

# In another terminal, monitor stats
watch -n1 'curl -s http://localhost:8000/api/websocket/stats'

# Open multiple connections and watch count
```

## ✅ **Summary**

### **Current Status:**

| Aspect | Status | Issue |
|--------|--------|-------|
| Socket cleanup | ⚠️ IMPLICIT | Relies on RAII, no explicit close |
| Broadcast RX cleanup | ⚠️ IMPLICIT | Auto-drop, no logging |
| Connection tracking | ❌ MISSING | No way to know active count |
| Timeout handling | ❌ MISSING | Zombie connections possible |
| Drop guards | ❌ MISSING | No guaranteed cleanup logging |
| Memory leak potential | ⚠️ MEDIUM | Under stress or errors |

### **Recommended Priority:**

1. **HIGH**: Add explicit socket.close() và cleanup logging
2. **HIGH**: Add connection tracking với AtomicUsize
3. **MEDIUM**: Add connection timeout (300s)
4. **MEDIUM**: Add Drop guard cho guaranteed cleanup
5. **LOW**: Add health check endpoint cho connection stats

### **Expected Impact:**

- ✅ **Better observability**: Know when connections cleanup
- ✅ **Prevent zombie connections**: Timeout mechanism
- ✅ **Track memory usage**: Connection counter
- ✅ **Easier debugging**: Explicit cleanup logs

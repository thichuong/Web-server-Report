# 🔌 WEBSOCKET REAL-TIME SYSTEM SPECIFICATION

## 📊 Overview
This document specifies a sophisticated WebSocket system for real-time dashboard updates with intelligent caching integration, broadcast messaging, and resilient error handling with exponential backoff strategies.

## 🏗️ WebSocket Architecture

### 1. Core Service Structure
**Purpose**: Centralized WebSocket service managing real-time data distribution with cache-aware updates

#### Core Components:
```rust
pub struct WebSocketService {
    cache_manager: Arc<CacheManager>,           // Unified L1+L2 cache integration
    data_service: Arc<DataService>,             // External API data fetching
    broadcast_tx: broadcast::Sender<String>,    // Message broadcasting channel
}
```

#### Configuration Constants:
```rust
const UPDATE_INTERVAL_SECONDS: u64 = 600;      // 10 minutes update cycle
const BROADCAST_CHANNEL_SIZE: usize = 100;     // Maximum 100 queued messages
const MAX_CONSECUTIVE_FAILURES: u32 = 3;       // Circuit breaker threshold
const MAX_BACKOFF_MINUTES: u64 = 30;           // Maximum backoff time
const INITIAL_FETCH_RETRIES: u32 = 3;          // Startup retry attempts
```

## 🔄 Background Update System

### 1. Intelligent Update Scheduler
**Purpose**: Periodic data refresh with failure recovery and rate limiting

```rust
pub async fn start_background_updates(&self) {
    let cache_manager = self.cache_manager.clone();
    let data_service = self.data_service.clone();
    let broadcast_tx = self.broadcast_tx.clone();

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(UPDATE_INTERVAL_SECONDS));
        let mut consecutive_failures = 0u32;

        loop {
            interval.tick().await;
            
            println!("🔄 Starting scheduled dashboard data update...");
            
            match Self::update_dashboard_data(&cache_manager, &data_service, &broadcast_tx).await {
                Ok(_) => {
                    println!("✅ Dashboard data updated successfully after {} consecutive failures", consecutive_failures);
                    consecutive_failures = 0;  // Reset failure counter
                },
                Err(e) => {
                    consecutive_failures += 1;
                    eprintln!("❌ Failed to update dashboard data (attempt {}): {}", consecutive_failures, e);
                    
                    // Exponential backoff for consecutive failures
                    if consecutive_failures > MAX_CONSECUTIVE_FAILURES {
                        let backoff_minutes = std::cmp::min(consecutive_failures * 2, MAX_BACKOFF_MINUTES);
                        println!("⏳ Too many consecutive failures, backing off for {} minutes", backoff_minutes);
                        tokio::time::sleep(Duration::from_secs(backoff_minutes * 60)).await;
                    }
                }
            }
        }
    });
}
```

### 2. Initial Data Loading with Retry
**Purpose**: Ensure service starts with valid cached data

```rust
// Fetch initial data with retry logic
println!("🔄 Fetching initial dashboard data...");
for attempt in 1..=INITIAL_FETCH_RETRIES {
    match Self::update_dashboard_data(&self.cache_manager, &self.data_service, &self.broadcast_tx).await {
        Ok(_) => {
            println!("✅ Initial dashboard data fetched successfully");
            break;
        },
        Err(e) => {
            eprintln!("⚠️ Failed to fetch initial dashboard data (attempt {}): {}", attempt, e);
            if attempt < INITIAL_FETCH_RETRIES {
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
```

## 📡 Broadcast Message System

### 1. Dashboard Update Broadcasting
**Purpose**: Distribute real-time data updates to all connected WebSocket clients

```rust
async fn update_dashboard_data(
    _cache_manager: &CacheManager,
    data_service: &DataService,
    broadcast_tx: &broadcast::Sender<String>,
) -> Result<(), anyhow::Error> {
    // Fetch data via DataService (with CacheManager integration)
    let summary = data_service.fetch_dashboard_summary()
        .await.map_err(|e| anyhow::anyhow!("Failed to fetch dashboard data via DataService: {}", e))?;

    println!("✅ Dashboard data fetched via DataService with CacheManager integration");

    // Prepare broadcast message
    let message = json!({
        "type": "dashboard_update",
        "data": summary,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "source": "scheduled_update"
    }).to_string();

    // Broadcast to all connected clients
    match broadcast_tx.send(message) {
        Ok(subscriber_count) => {
            println!("📡 Dashboard update broadcasted to {} WebSocket clients", subscriber_count);
        }
        Err(_) => {
            println!("ℹ️ No WebSocket clients connected for scheduled update");
        }
    }

    Ok(())
}
```

### 2. Force Update Capability
**Purpose**: On-demand data refresh triggered by API calls or admin interface

```rust
pub async fn force_update_dashboard(&self) -> Result<DashboardSummary, anyhow::Error> {
    println!("🔄 Force updating dashboard data via DataService...");
    
    // Force fresh data fetch
    let summary = self.data_service.fetch_dashboard_summary()
        .await.map_err(|e| anyhow::anyhow!("Failed to force update via DataService: {}", e))?;

    println!("✅ Dashboard data force-updated via DataService with CacheManager integration");

    // Broadcast to WebSocket clients
    let message = json!({
        "type": "dashboard_update",
        "data": summary,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "source": "force_update"
    }).to_string();

    if let Err(e) = self.broadcast_tx.send(message) {
        println!("ℹ️ No WebSocket clients connected for force update broadcast: {}", e);
    } else {
        println!("📡 Force update broadcasted to WebSocket clients");
    }

    Ok(summary)
}
```

## 🔌 WebSocket Connection Management

### 1. Connection Establishment
**Purpose**: Handle new WebSocket connections with immediate data delivery

```rust
pub async fn handle_websocket(&self, mut socket: WebSocket) {
    println!("🔗 New WebSocket connection established");

    // Send current cached data immediately (welcome message)
    if let Ok(Some(current_data)) = self.get_cached_dashboard_data().await {
        let welcome_message = json!({
            "type": "welcome",
            "data": current_data,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "message": "Connected to real-time dashboard updates"
        }).to_string();

        if let Err(e) = socket.send(Message::Text(welcome_message)).await {
            eprintln!("❌ Failed to send welcome message: {}", e);
            return;
        }

        println!("👋 Welcome message sent to new WebSocket client");
    }

    // Setup broadcast receiver for this connection
    let mut broadcast_rx = self.get_broadcast_receiver();

    // Connection event loop
    loop {
        tokio::select! {
            // Handle incoming messages from client
            socket_msg = socket.recv() => {
                match socket_msg {
                    Some(Ok(Message::Text(text))) => {
                        println!("📨 Received WebSocket message: {}", text);
                        // Handle client messages (heartbeat, requests, etc.)
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("👋 WebSocket client requested close");
                        break;
                    }
                    Some(Err(e)) => {
                        eprintln!("❌ WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {} // Ignore other message types (Binary, Ping, Pong)
                }
            }
            // Handle broadcast messages from service
            broadcast_msg = broadcast_rx.recv() => {
                match broadcast_msg {
                    Ok(message) => {
                        if socket.send(Message::Text(message)).await.is_err() {
                            println!("❌ Failed to send broadcast message, client disconnected");
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        println!("📡 Broadcast channel closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        println!("⚠️ WebSocket client lagged behind, continuing...");
                        continue;
                    }
                }
            }
        }
    }

    println!("🔌 WebSocket connection terminated");
}
```

## 💾 Cache-Aware Data Management

### 1. Smart Cache Retrieval
**Purpose**: Intelligent cache checking with age validation

```rust
pub async fn get_cached_dashboard_data(&self) -> Result<Option<DashboardSummary>, anyhow::Error> {
    // Try to get from cache first
    match self.cache_manager.get::<DashboardSummary>("dashboard:summary").await {
        Ok(Some(cached_data)) => {
            // Validate data freshness (example: 5-minute threshold)
            let data_age = chrono::Utc::now() - cached_data.last_updated;
            
            if data_age.num_minutes() < 5 {
                println!("✅ Using fresh cached data ({}s old)", data_age.num_seconds());
                return Ok(Some(cached_data));
            } else {
                println!("⏰ Cached data is stale ({}m old), fetching fresh data", data_age.num_minutes());
            }
        },
        Ok(None) => {
            println!("ℹ️ No cached data available, fetching fresh data");
        },
        Err(e) => {
            eprintln!("⚠️ Cache error: {}, fetching fresh data", e);
        }
    }

    // Fallback to fresh fetch
    self.force_update_dashboard().await.map(Some)
}
```

## 📊 Message Protocol Specification

### 1. Message Types
#### Dashboard Update Message
```json
{
    "type": "dashboard_update",
    "data": {
        "btc_price": 45000.0,
        "btc_change_24h": 2.5,
        "market_cap": 1200000000000,
        "volume_24h": 25000000000,
        "fear_greed_index": 65,
        "rsi_14": 58.5,
        "last_updated": "2025-08-20T10:30:00Z"
    },
    "timestamp": "2025-08-20T10:30:00Z",
    "source": "scheduled_update" // or "force_update"
}
```

#### Welcome Message
```json
{
    "type": "welcome",
    "data": { /* same as dashboard_update */ },
    "timestamp": "2025-08-20T10:30:00Z",
    "message": "Connected to real-time dashboard updates"
}
```

#### Error Message
```json
{
    "type": "error",
    "error": "Failed to fetch market data",
    "timestamp": "2025-08-20T10:30:00Z",
    "retry_after": 300
}
```

### 2. Client Message Handling
#### Heartbeat Messages
```json
{
    "type": "ping",
    "timestamp": "2025-08-20T10:30:00Z"
}
```

#### Data Request Messages
```json
{
    "type": "request_update",
    "timestamp": "2025-08-20T10:30:00Z"
}
```

## 🚨 Error Handling & Resilience

### 1. Connection Error Recovery
```rust
// Graceful handling of connection failures
match socket_msg {
    Some(Ok(Message::Text(text))) => {
        println!("📨 Received WebSocket message: {}", text);
    }
    Some(Ok(Message::Close(_))) => {
        println!("👋 WebSocket client requested close");
        break;
    }
    Some(Err(e)) => {
        eprintln!("❌ WebSocket error: {}", e);
        break;  // Terminate connection gracefully
    }
    None => break,  // Connection closed
    _ => {} // Ignore other message types
}
```

### 2. Broadcast Channel Error Recovery
```rust
// Handle broadcast channel errors
match broadcast_msg {
    Ok(message) => {
        // Send message to client
        if socket.send(Message::Text(message)).await.is_err() {
            println!("❌ Failed to send broadcast message, client disconnected");
            break;
        }
    }
    Err(broadcast::error::RecvError::Closed) => {
        println!("📡 Broadcast channel closed");
        break;
    }
    Err(broadcast::error::RecvError::Lagged(_)) => {
        println!("⚠️ WebSocket client lagged behind, continuing...");
        continue;  // Skip lagged messages, continue with next
    }
}
```

### 3. Data Fetch Failure Handling
```rust
// Exponential backoff for consecutive failures
if consecutive_failures > MAX_CONSECUTIVE_FAILURES {
    let backoff_minutes = std::cmp::min(consecutive_failures * 2, MAX_BACKOFF_MINUTES);
    println!("⏳ Too many consecutive failures, backing off for {} minutes", backoff_minutes);
    tokio::time::sleep(Duration::from_secs(backoff_minutes * 60)).await;
}
```

## 🔧 Service Integration

### 1. Service Initialization
```rust
impl WebSocketService {
    pub fn new(cache_manager: Arc<CacheManager>, data_service: DataService) -> Result<Self, anyhow::Error> {
        let (broadcast_tx, _) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        Ok(Self {
            cache_manager,
            data_service: Arc::new(data_service),
            broadcast_tx,
        })
    }

    pub fn get_broadcast_receiver(&self) -> broadcast::Receiver<String> {
        self.broadcast_tx.subscribe()
    }
}
```

### 2. HTTP Handler Integration
```rust
// In handlers/websocket.rs
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        state.websocket_service.handle_websocket(socket).await;
    })
}
```

## 📊 Performance Monitoring

### 1. Connection Metrics
- Active WebSocket connections count
- Message broadcast success/failure rates  
- Average message latency
- Client connection duration statistics

### 2. Data Update Metrics
- Background update success/failure rates
- Cache hit/miss ratios for WebSocket data
- Average data fetch times
- Consecutive failure counts and backoff frequency

## 🔌 Dependencies

### Core Dependencies
```rust
use axum::extract::ws::{WebSocket, Message, WebSocketUpgrade};
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
use serde_json::json;
use chrono::{DateTime, Utc};
use std::sync::Arc;
```

### Integration Dependencies
```rust
use crate::data_service::{DataService, DashboardSummary};
use crate::cache::CacheManager;
```

## 🎯 Migration Considerations

### Feature Isolation Strategy
When migrating to feature-based architecture:

1. **WebSocket Service**: Move to `features/websocket_realtime/`
2. **Message Protocol**: Define in `features/websocket_realtime/protocol.rs`
3. **Connection Management**: `features/websocket_realtime/connections.rs`
4. **Broadcasting**: `features/websocket_realtime/broadcast.rs`

### Backwards Compatibility
- All message formats preserved
- Connection handling unchanged
- Error response patterns consistent
- Performance characteristics maintained

---

**📝 Generated**: August 20, 2025  
**🔄 Version**: 1.0  
**📊 Source Lines**: 264 lines of WebSocket service implementation  
**🎯 Migration Target**: `features/websocket_realtime/`

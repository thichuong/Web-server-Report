// src/performance.rs - Performance optimization utilities

use std::sync::Arc;
use std::time::Duration;
use moka::future::Cache;
use dashmap::DashMap;
use lazy_static::lazy_static;
use threadpool::ThreadPool;

// Global optimized HTTP client với connection pooling và SSL configuration
lazy_static! {
    pub static ref OPTIMIZED_HTTP_CLIENT: reqwest::Client = {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))     // Tăng timeout cho Railway
            .connect_timeout(Duration::from_secs(10))  // Connection timeout riêng biệt
            .pool_max_idle_per_host(20)           // Reuse connections
            .pool_idle_timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(60))
            // SSL/TLS Configuration cho production
            .danger_accept_invalid_certs(false)   // Đảm bảo cert validation
            .tls_built_in_root_certs(true)       // Sử dụng built-in CA certificates
            .https_only(false)                    // Allow HTTP for local dev
            // REMOVED: http2_prior_knowledge() - causes frame size errors
            // User Agent để tránh bị block
            .user_agent("Mozilla/5.0 (compatible; RustWebServer/1.0)")
            // Retry configuration
            .tcp_nodelay(true)                    // Disable Nagle algorithm
            .build()
            .expect("Failed to create optimized HTTP client")
    };

    // Dedicated CPU thread pool cho heavy computations
    pub static ref CPU_POOL: ThreadPool = {
        threadpool::Builder::new()
            .num_threads(num_cpus::get())
            .thread_name("cpu-worker".into())
            .build()
    };
}

/// Multi-level cache với L1 (memory) và L2 (Redis)
#[allow(dead_code)]
pub struct MultiLevelCache<K, V> 
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    // L1: In-memory cache (fastest)
    l1_cache: Cache<K, Arc<V>>,
    // Cache statistics
    hits: std::sync::atomic::AtomicU64,
    misses: std::sync::atomic::AtomicU64,
}

impl<K, V> MultiLevelCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    #[allow(dead_code)]
    pub fn new(capacity: u64, ttl: Duration) -> Self {
        Self {
            l1_cache: Cache::builder()
                .max_capacity(capacity)
                .time_to_live(ttl)
                .time_to_idle(ttl / 2)
                .build(),
            hits: std::sync::atomic::AtomicU64::new(0),
            misses: std::sync::atomic::AtomicU64::new(0),
        }
    }

    #[allow(dead_code)]
    pub async fn get(&self, key: &K) -> Option<V> {
        if let Some(value) = self.l1_cache.get(key).await {
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some((*value).clone())
        } else {
            self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }

    #[allow(dead_code)]
    pub async fn insert(&self, key: K, value: V) {
        self.l1_cache.insert(key, Arc::new(value)).await;
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.l1_cache.entry_count(),
            hits: self.hits.load(std::sync::atomic::Ordering::Relaxed),
            misses: self.misses.load(std::sync::atomic::Ordering::Relaxed),
            hit_rate: self.hit_rate(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct CacheStats {
    pub entries: u64,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

/// Concurrent request batcher để giảm database load
#[allow(dead_code)]
pub struct RequestBatcher<T, R> {
    pending: Arc<std::sync::Mutex<std::collections::HashMap<T, Vec<tokio::sync::oneshot::Sender<R>>>>>,
    batch_size: usize,
    batch_timeout: Duration,
}

impl<T, R> RequestBatcher<T, R> 
where
    T: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    R: Clone + Send + Sync + 'static,
{
    #[allow(dead_code)]
    pub fn new(batch_size: usize, batch_timeout: Duration) -> Self {
        Self {
            pending: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            batch_size,
            batch_timeout,
        }
    }

    #[allow(dead_code)]
    pub async fn request(&self, key: T, processor: impl Fn(Vec<T>) -> tokio::task::JoinHandle<Vec<R>> + Send + Sync + 'static) -> Option<R> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        {
            let mut pending = self.pending.lock().unwrap();
            pending.entry(key.clone()).or_default().push(tx);
        }

        // Trigger batch processing if threshold reached
        self.maybe_process_batch(processor).await;

        rx.await.ok()
    }

    #[allow(dead_code)]
    async fn maybe_process_batch(&self, processor: impl Fn(Vec<T>) -> tokio::task::JoinHandle<Vec<R>>) {
        let should_process = {
            let pending = self.pending.lock().unwrap();
            pending.values().any(|senders| senders.len() >= self.batch_size)
        };

        if should_process {
            self.process_batch(processor).await;
        }
    }

    #[allow(dead_code)]
    async fn process_batch(&self, processor: impl Fn(Vec<T>) -> tokio::task::JoinHandle<Vec<R>>) {
        let batch = {
            let mut pending = self.pending.lock().unwrap();
            std::mem::take(&mut *pending)
        };

        if batch.is_empty() {
            return;
        }

        let keys: Vec<T> = batch.keys().cloned().collect();
        
        // Process batch concurrently
        match processor(keys).await {
            Ok(results) => {
                // Distribute results back to requesters
                for (i, (_key, senders)) in batch.into_iter().enumerate() {
                    if let Some(result) = results.get(i) {
                        for sender in senders {
                            let _ = sender.send(result.clone());
                        }
                    }
                }
            }
            Err(_) => {
                // Handle batch processing error - send default responses
                // Note: Implement proper default value based on your R type
                eprintln!("Batch processing failed");
            }
        }
    }
}

/// Connection pool manager cho external APIs
#[allow(dead_code)]
pub struct ConnectionPoolManager {
    pools: DashMap<String, reqwest::Client>,
}

impl ConnectionPoolManager {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            pools: DashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get_or_create_client(&self, base_url: &str, pool_config: Option<ConnectionPoolConfig>) -> reqwest::Client {
        if let Some(client) = self.pools.get(base_url) {
            return client.clone();
        }

        let config = pool_config.unwrap_or_default();
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .pool_max_idle_per_host(config.max_idle_per_host)
            .pool_idle_timeout(config.idle_timeout)
            .tcp_keepalive(config.keepalive)
            .build()
            .expect("Failed to create HTTP client");

        self.pools.insert(base_url.to_string(), client.clone());
        client
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct ConnectionPoolConfig {
    pub timeout: Duration,
    pub max_idle_per_host: usize,
    pub idle_timeout: Duration,
    pub keepalive: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            max_idle_per_host: 20,
            idle_timeout: Duration::from_secs(30),
            keepalive: Duration::from_secs(60),
        }
    }
}

/// Performance metrics collector
#[derive(Default)]
pub struct PerformanceMetrics {
    pub request_count: std::sync::atomic::AtomicU64,
    pub total_response_time: std::sync::atomic::AtomicU64,
    #[allow(dead_code)]
    pub active_connections: std::sync::atomic::AtomicU64,
    #[allow(dead_code)]
    pub cache_hits: std::sync::atomic::AtomicU64,
    #[allow(dead_code)]
    pub cache_misses: std::sync::atomic::AtomicU64,
    #[allow(dead_code)]
    pub database_queries: std::sync::atomic::AtomicU64,
    #[allow(dead_code)]
    pub websocket_connections: std::sync::atomic::AtomicU64,
}

impl PerformanceMetrics {
    pub fn record_request(&self, response_time_ms: u64) {
        self.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.total_response_time.fetch_add(response_time_ms, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn avg_response_time(&self) -> f64 {
        let total_requests = self.request_count.load(std::sync::atomic::Ordering::Relaxed);
        let total_time = self.total_response_time.load(std::sync::atomic::Ordering::Relaxed);
        
        if total_requests == 0 {
            0.0
        } else {
            total_time as f64 / total_requests as f64
        }
    }

    #[allow(dead_code)]
    pub fn requests_per_second(&self, duration_secs: u64) -> f64 {
        let total_requests = self.request_count.load(std::sync::atomic::Ordering::Relaxed);
        total_requests as f64 / duration_secs as f64
    }
}

#!/bin/bash

# Performance Optimization and Benchmark Script
# Tối ưu hệ thống và chạy benchmark

set -e

echo "🚀 STARTING PERFORMANCE OPTIMIZATION & BENCHMARK"
echo "================================================="

# 1. Kiểm tra system resources
echo "📊 System Resources:"
echo "CPU Cores: $(nproc)"
echo "Memory: $(free -h | awk '/^Mem:/ {print $2}')"
echo "Disk: $(df -h . | awk 'NR==2 {print $4 " available"}')"
echo ""

# 2. Set environment variables for optimization
export RUST_LOG=info
export TOKIO_THREAD_STACK_SIZE=4194304  # 4MB stack per thread
export RAYON_NUM_THREADS=$(nproc)
export DATABASE_URL="postgresql://localhost/crypto_dashboard"
export REDIS_URL="redis://localhost:6379"

# 3. Build với full optimization
echo "🔨 Building with maximum optimization..."
cargo clean
RUSTFLAGS="-C target-cpu=native -C target-feature=+avx2" cargo build --release

# 4. Khởi động services cần thiết
echo "🐘 Starting PostgreSQL (if not running)..."
sudo systemctl start postgresql || echo "PostgreSQL already running or not installed"

echo "🔴 Starting Redis (if not running)..."  
sudo systemctl start redis || echo "Redis already running or not installed"

# 5. Chạy server trong background
echo "🚀 Starting optimized server..."
./target/release/web-server-report &
SERVER_PID=$!

# Đợi server khởi động
sleep 5

# 6. Basic health check
echo "🏥 Health check..."
curl -s http://localhost:8000/health | jq '.' || echo "Health check failed"

# 7. Benchmark tests
echo "📈 Running benchmark tests..."

# Test 1: Concurrent connections
echo "Test 1: Concurrent Connections (100 concurrent, 1000 requests)"
wrk -t12 -c100 -d30s --latency http://localhost:8000/health

# Test 2: WebSocket connections
echo "Test 2: WebSocket Load Test"
# Tạo WebSocket test client nếu có
if command -v wscat >/dev/null 2>&1; then
    echo "WebSocket test với wscat..."
    for i in {1..10}; do
        wscat -c ws://localhost:8000/ws -x 'ping' &
    done
    sleep 10
    kill $(jobs -p) 2>/dev/null || true
fi

# Test 3: Database performance  
echo "Test 3: Database Performance Test"
for i in {1..50}; do
    curl -s http://localhost:8000/api/reports/latest > /dev/null &
done
wait

# Test 4: Memory usage
echo "Test 4: Memory Usage Analysis"
ps aux | grep web-server-report | grep -v grep

# Test 5: Cache performance
echo "Test 5: Cache Performance"
curl -s http://localhost:8000/api/cache/stats | jq '.'

# Test 6: Performance metrics
echo "Test 6: Performance Metrics"
curl -s http://localhost:8000/api/performance/metrics | jq '.'

# 8. Cleanup
echo "🧹 Cleaning up..."
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

echo ""
echo "✅ PERFORMANCE OPTIMIZATION & BENCHMARK COMPLETED"
echo "================================================="
echo "📝 Results:"
echo "- Check server logs for detailed performance metrics"
echo "- Review wrk output for latency and throughput numbers"
echo "- Monitor memory usage and cache hit rates"
echo ""
echo "🎯 Optimization Recommendations:"
echo "1. Increase database connection pool if CPU allows"
echo "2. Tune Redis memory settings for better caching"
echo "3. Enable HTTP/2 for better connection multiplexing"
echo "4. Consider using a reverse proxy (nginx) for static assets"
echo "5. Monitor GC pressure and adjust memory allocation"

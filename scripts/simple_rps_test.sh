#!/bin/bash

# Simple RPS benchmark script
SERVER_URL="http://localhost:8050"
ENDPOINT="/crypto_report"

echo "🚀 Simple RPS Benchmark"
echo "======================="
echo "Server: $SERVER_URL"
echo "Endpoint: $ENDPOINT"
echo "CPUs: $(nproc) cores"
echo ""

# Check server health
echo "🔍 Checking server health..."
if ! curl -s "${SERVER_URL}/health" > /dev/null; then
    echo "❌ Server không phản hồi"
    exit 1
fi
echo "✅ Server OK"
echo ""

# Function to run RPS test
run_rps_test() {
    local test_name=$1
    local concurrent_users=$2
    local requests_per_user=$3
    local total_requests=$((concurrent_users * requests_per_user))
    
    echo "📊 $test_name"
    echo "   Users: $concurrent_users, Requests/user: $requests_per_user, Total: $total_requests"
    
    # Record start time
    local start_time=$(date +%s)
    
    # Run concurrent requests
    for i in $(seq 1 $concurrent_users); do
        {
            for j in $(seq 1 $requests_per_user); do
                curl -s "${SERVER_URL}${ENDPOINT}" > /dev/null 2>&1
            done
        } &
    done
    
    # Wait for all background processes
    wait
    
    # Record end time
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    if [ $duration -eq 0 ]; then
        duration=1  # Prevent division by zero
    fi
    
    local rps=$((total_requests / duration))
    
    echo "   Duration: ${duration}s"
    echo "   RPS: $rps requests/second"
    echo "   Avg latency: $((duration * 1000 / total_requests))ms per request"
    echo ""
}

# Sequential baseline test
echo "📈 Sequential Baseline Test (10 requests)"
start_time=$(date +%s%N)
for i in {1..10}; do
    curl -s "${SERVER_URL}${ENDPOINT}" > /dev/null
done
end_time=$(date +%s%N)
duration_ms=$(((end_time - start_time) / 1000000))
echo "   Total time: ${duration_ms}ms"
echo "   Avg per request: $((duration_ms / 10))ms"
echo ""

# RPS tests
run_rps_test "Light Load Test" 50 100
run_rps_test "Medium Load Test" 100 200
run_rps_test "Heavy Load Test" 200 250
run_rps_test "Extreme Load Test" 500 100

# Get final metrics
echo "📊 Final Server Metrics:"
if final_metrics=$(curl -s "${SERVER_URL}/metrics" 2>/dev/null); then
    echo "$final_metrics" | jq -r '.performance.total_requests_processed' 2>/dev/null || echo "Parse error"
else
    echo "Could not fetch metrics"
fi

echo ""
echo "🎯 Cache Performance Test"
echo "========================"

# Cache miss test
echo "Cache miss (first unique request):"
time curl -s "${SERVER_URL}/crypto_report/999" > /dev/null

# Cache hit test
echo "Cache hit (repeated request):"
time curl -s "${SERVER_URL}${ENDPOINT}" > /dev/null
time curl -s "${SERVER_URL}${ENDPOINT}" > /dev/null

echo ""
echo "✅ RPS Benchmark completed!"

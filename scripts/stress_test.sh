#!/bin/bash

# Stress test script cho high-concurrency server
# Test crypto_index endpoint với nhiều concurrent requests

SERVER_URL="http://localhost:8050"
ENDPOINT="/crypto_report"
CONCURRENT_USERS=50
DURATION=30
RAMP_UP=5

echo "🧪 Starting stress test for crypto dashboard server"
echo "📊 Target: ${SERVER_URL}${ENDPOINT}"
echo "👥 Concurrent users: ${CONCURRENT_USERS}"
echo "⏱️  Duration: ${DURATION}s"
echo "📈 Ramp up: ${RAMP_UP}s"
echo ""

# Check if server is running
echo "🔍 Checking server health..."
if ! curl -s "${SERVER_URL}/health" > /dev/null; then
    echo "❌ Server is not responding at ${SERVER_URL}"
    echo "   Please start the server first with: cargo run"
    exit 1
fi

echo "✅ Server is healthy"
echo ""

# Function to make concurrent requests
make_requests() {
    local user_id=$1
    local requests_per_user=10
    local success_count=0
    
    for i in $(seq 1 $requests_per_user); do
        start_time=$(date +%s%N)
        
        # Make request with timeout
        if response=$(curl -s -w "%{http_code}" -m 10 "${SERVER_URL}${ENDPOINT}" -o /dev/null); then
            end_time=$(date +%s%N)
            duration=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
            
            if [ "$response" = "200" ]; then
                success_count=$((success_count + 1))
                echo "✅ User$user_id-Req$i: ${duration}ms [200]"
            else
                echo "❌ User$user_id-Req$i: ${duration}ms [$response]"
            fi
        else
            echo "💥 User$user_id-Req$i: TIMEOUT or CONNECTION ERROR"
        fi
        
        # Small delay between requests
        sleep 0.1
    done
    
    echo "📊 User$user_id completed: $success_count/$requests_per_user successful"
}

# Start concurrent users
echo "🚀 Starting concurrent load test..."
echo "Date: $(date)"
echo ""

# Get initial metrics
echo "📊 Initial server metrics:"
curl -s "${SERVER_URL}/metrics" | jq -r '.performance.total_requests_processed' 2>/dev/null || echo "N/A"
echo ""

pids=()

# Start all users
for i in $(seq 1 $CONCURRENT_USERS); do
    make_requests $i &
    pids+=($!)
    
    # Ramp up gradually
    if [ $((i % 10)) -eq 0 ]; then
        echo "👥 Started $i users..."
        sleep 1
    fi
done

echo "⏳ All $CONCURRENT_USERS users started. Waiting for completion..."

# Wait for all users to complete
for pid in "${pids[@]}"; do
    wait $pid
done

echo ""
echo "🎉 Load test completed!"

# Get final metrics
echo "📊 Final server metrics:"
if final_metrics=$(curl -s "${SERVER_URL}/metrics" 2>/dev/null); then
    echo "$final_metrics" | jq '.' 2>/dev/null || echo "$final_metrics"
else
    echo "❌ Could not fetch final metrics"
fi

echo ""
echo "✨ Test completed at $(date)"

# Additional performance check
echo ""
echo "🔍 Quick performance check (5 sequential requests):"
for i in {1..5}; do
    start_time=$(date +%s%N)
    response=$(curl -s -w "%{http_code}" "${SERVER_URL}${ENDPOINT}" -o /dev/null)
    end_time=$(date +%s%N)
    duration=$(( (end_time - start_time) / 1000000 ))
    
    cache_header=$(curl -s -I "${SERVER_URL}${ENDPOINT}" | grep -i "x-cache" | cut -d' ' -f2- | tr -d '\r\n')
    
    echo "Request $i: ${duration}ms [$response] Cache: ${cache_header:-'N/A'}"
done

echo ""
echo "✅ Stress test completed successfully!"

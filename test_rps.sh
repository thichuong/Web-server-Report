#!/bin/bash

# Web Server RPS Benchmark Script
# Uses Apache Benchmark (ab) to test server performance.

# Configuration
URL=${1:-"http://localhost:8000/"}
CONCURRENCY=${2:-500}
NUM_REQUESTS=${3:-50000}

# Check if ab is installed
if ! command -v ab &> /dev/null; then
    echo "Error: 'ab' (Apache Benchmark) is not installed."
    echo "Please install it using your package manager:"
    echo "  - Debian/Ubuntu: sudo apt install apache2-utils"
    echo "  - RedHat/Fedora/CentOS: sudo dnf install httpd-tools"
    exit 1
fi

# Check if server is running
if ! curl -s --head --request GET "$URL" | grep "200 OK" > /dev/null; then
    echo "Warning: Server at $URL might not be reachable. Benchmarking anyway..."
fi

echo "--------------------------------------------------------"
echo "Starting benchmark for $URL"
echo "Requests: $NUM_REQUESTS, Concurrency: $CONCURRENCY"
echo "--------------------------------------------------------"

# Run Apache Benchmark
# -n: Number of requests to perform
# -c: Number of multiple requests to make at a time
# -k: Use HTTP KeepAlive feature
ab -n "$NUM_REQUESTS" -c "$CONCURRENCY" -k "$URL"

echo "--------------------------------------------------------"
echo "Benchmark completed."

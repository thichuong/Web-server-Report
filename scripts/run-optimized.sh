#!/bin/bash

# Quick development run script with optimizations
# Tối ưu cargo run cho development với performance tốt

set -e

echo "🚀 Starting optimized development server..."
echo "========================================="

# Set environment variables for optimal development
export RUST_LOG=info
export TOKIO_THREAD_STACK_SIZE=4194304  # 4MB stack per thread
export RAYON_NUM_THREADS=$(nproc)

# Check if database and redis are running
echo "🔍 Checking services..."
if ! pgrep -x "postgres" > /dev/null; then
    echo "⚠️  PostgreSQL is not running. Starting it..."
    sudo systemctl start postgresql 2>/dev/null || echo "Please start PostgreSQL manually"
fi

if ! pgrep -x "redis-server" > /dev/null; then
    echo "⚠️  Redis is not running. Starting it..."
    sudo systemctl start redis 2>/dev/null || echo "Please start Redis manually"
fi

# Set development-friendly Rust flags
export RUSTFLAGS="-C target-cpu=native -C target-feature=+crt-static"

echo "📊 System info:"
echo "CPU cores: $(nproc)"
echo "Available memory: $(free -h | awk '/^Mem:/ {print $7}')"
echo ""

echo "🔧 Building with optimized dev profile..."

# Use cargo run with optimized settings
exec cargo run \
    --config "profile.dev.opt-level=1" \
    --config "profile.dev.codegen-units=8" \
    "$@"

#!/bin/bash

# Development runner với optimizations
# Sử dụng: ./dev.sh hoặc ./dev.sh --release

set -e

# Default to development mode
MODE="dev"
if [[ "$1" == "--release" ]]; then
    MODE="release"
    shift
fi

echo "🚀 Starting $MODE server..."

# Set optimization environment variables
export RUST_LOG=info
export TOKIO_THREAD_STACK_SIZE=4194304  
export RAYON_NUM_THREADS=$(nproc)

# RUSTFLAGS for better performance
export RUSTFLAGS="-C target-cpu=native"

if [[ "$MODE" == "release" ]]; then
    echo "🔨 Building and running release version..."
    exec cargo run --release "$@"
else
    echo "🔧 Running optimized development version..."
    # Run with optimized dev profile
    exec cargo run "$@"
fi

#!/bin/bash
#
# Test script for Finnhub US Stock Indices Integration
#
echo "📈 Testing Finnhub US Stock Market Indices Integration"
echo "============================================================"

# Check if Finnhub API key is set
if [ -z "$FINNHUB_API_KEY" ]; then
    echo "⚠️ Warning: FINNHUB_API_KEY not set"
    echo "   You can get a free API key at: https://finnhub.io/"
    echo "   Then run: export FINNHUB_API_KEY='your_api_key_here'"
    echo ""
else
    echo "🔑 Finnhub API key found: ${FINNHUB_API_KEY:0:8}..."
fi

# Check if CoinMarketCap API key is set
if [ -z "$CMC_API_KEY" ]; then
    echo "ℹ️ Info: CMC_API_KEY not set (optional for crypto fallback)"
else
    echo "🔑 CoinMarketCap API key found: ${CMC_API_KEY:0:8}..."
fi

echo ""
echo "🔧 Building project..."
if ! cargo build --example test_finnhub_integration; then
    echo "❌ Build failed"
    exit 1
fi

echo ""
echo "🚀 Running Finnhub integration test..."
echo ""

# Run the example
cargo run --example test_finnhub_integration

echo ""
echo "📊 Test completed!"
echo ""
echo "💡 Notes:"
echo "  • If US indices show 'unknown' status, check your Finnhub API key"
echo "  • Crypto data should work even without CMC_API_KEY (fallback only)"
echo "  • All APIs have different rate limits - check documentation if issues occur"

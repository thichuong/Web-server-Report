#!/bin/bash

# Test script to verify rate limiting improvements
echo "🔄 Testing Rate Limiting Improvements"
echo "=====================================."

# Check if Redis is running (optional - server should handle Redis failures gracefully)
echo "🔍 Checking Redis connection..."
redis-cli ping 2>/dev/null && echo "✅ Redis is running" || echo "⚠️ Redis not available (server should handle this gracefully)"

echo ""
echo "📊 Starting server to test improvements..."
echo "The server should now:"
echo "  - Update dashboard data every 10 minutes (instead of 5)"
echo "  - Cache data in Redis with 1-hour TTL"
echo "  - Use intelligent fallback when cache is available"
echo "  - Handle 429 errors with longer retry delays"
echo ""
echo "🚀 To start the server, run:"
echo "   cargo run --release"
echo ""
echo "📡 To test the API endpoints:"
echo "   curl http://localhost:3000/api/dashboard/summary"
echo "   curl -X POST http://localhost:3000/api/dashboard/force-refresh"
echo ""
echo "🔍 Watch logs for these indicators:"
echo "   ✅ Using fresh cached data (Xm old)"
echo "   🔄 Fetching fresh dashboard data..."
echo "   ✅ Dashboard data cached to Redis with TTL: 3600s"
echo "   ⏳ Retry X/Y after Zs for error: ..."

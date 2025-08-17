#!/bin/bash

# Database connection test script
echo "🔍 Testing database connectivity..."
echo "=================================="

# Load environment variables
source .env

# Test basic connectivity
echo "📡 Testing network connectivity to database host..."
echo "Database URL: $DATABASE_URL"

# Extract host and port from DATABASE_URL
HOST=$(echo $DATABASE_URL | grep -oP '(?<=@)[^:/]+(?=:)')
PORT=$(echo $DATABASE_URL | grep -oP '(?<=:)[0-9]+(?=/)')

echo "Host: $HOST"
echo "Port: $PORT"

# Test if host is reachable
if command -v nc >/dev/null 2>&1; then
    echo "🔍 Testing TCP connection..."
    if timeout 10 nc -z $HOST $PORT; then
        echo "✅ TCP connection to $HOST:$PORT successful"
    else
        echo "❌ Cannot reach $HOST:$PORT"
        echo "This might be why the database connection is failing."
        echo ""
        echo "💡 Solutions:"
        echo "1. Check if you're connected to the internet"
        echo "2. The Railway database might be sleeping (free tier limitation)"
        echo "3. Try using a local PostgreSQL database for development"
        echo ""
        echo "🚀 To set up local PostgreSQL:"
        echo "   sudo systemctl start postgresql"
        echo "   sudo -u postgres createuser -s $USER"
        echo "   sudo -u postgres createdb report_db"
        echo "   # Update .env with: DATABASE_URL=postgresql://localhost/report_db"
    fi
else
    echo "⚠️ netcat (nc) not available, skipping TCP test"
fi

echo ""
echo "🔧 Alternative: Use local PostgreSQL for development"
echo "   1. Install PostgreSQL: sudo dnf install postgresql postgresql-server"
echo "   2. Initialize: sudo postgresql-setup --initdb"
echo "   3. Start service: sudo systemctl start postgresql"
echo "   4. Create database: sudo -u postgres createdb report_db"
echo "   5. Update DATABASE_URL in .env to: postgresql://localhost/report_db"

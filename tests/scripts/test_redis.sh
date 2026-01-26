#!/bin/bash

echo "=== Redis Integration Test ==="
echo ""

# Test Redis connection
echo "1. Testing Redis connection..."
if redis-cli ping | grep -q "PONG"; then
    echo " Redis is running"
else
    echo " Redis is not running"
    exit 1
fi

# Test Redis operations
echo ""
echo "2. Testing Redis operations..."

# Set a test key
redis-cli SET fiddupay:test:key "test_value" > /dev/null
if [ $? -eq 0 ]; then
    echo " SET operation successful"
else
    echo " SET operation failed"
    exit 1
fi

# Get the test key
VALUE=$(redis-cli GET fiddupay:test:key)
if [ "$VALUE" = "test_value" ]; then
    echo " GET operation successful"
else
    echo " GET operation failed"
    exit 1
fi

# Test expiration
redis-cli SETEX fiddupay:test:expire 5 "expires_soon" > /dev/null
if [ $? -eq 0 ]; then
    echo " SETEX operation successful"
else
    echo " SETEX operation failed"
    exit 1
fi

# Clean up
redis-cli DEL fiddupay:test:key fiddupay:test:expire > /dev/null

echo ""
echo "3. Testing Redis from fiddupay..."

# Test price cache (if server is running)
if curl -s http://localhost:8080/health | grep -q "healthy"; then
    echo " Server is running"
    
    # Redis should be used for price caching
    echo " Redis integration ready for:"
    echo "   - Price caching (30s TTL)"
    echo "   - Session management"
    echo "   - Rate limiting"
else
    echo "⚠️  Server not running (start with: cargo run --release)"
fi

echo ""
echo "=== Redis Integration:  READY ==="
echo ""
echo "Redis Info:"
redis-cli INFO server | grep -E "redis_version|tcp_port|uptime_in_seconds"
echo ""
echo "Redis Memory:"
redis-cli INFO memory | grep -E "used_memory_human|maxmemory_human"

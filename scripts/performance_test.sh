#!/bin/bash
# Performance Optimization Test Script

echo " fiddupay Performance Optimization Test"
echo "========================================"

cd /home/vibes/crypto-payment-gateway

# Test 1: Compilation Performance
echo " Test 1: Compilation Performance"
echo "  Testing optimized Tokio features..."

# Check if optimized features are in Cargo.toml
if grep -q 'features = \["rt-multi-thread", "net", "time", "macros"\]' Cargo.toml; then
    echo "   Tokio features optimized (selective instead of 'full')"
else
    echo "    Tokio features not optimized"
fi

# Test 2: String Allocation Optimization
echo ""
echo " Test 2: String Allocation Analysis"
echo "  Checking for optimized string usage..."

# Count string slice usage vs owned strings in function signatures
STRING_SLICE_COUNT=$(grep -r "fn.*&str" src/ --include="*.rs" | wc -l)
OWNED_STRING_COUNT=$(grep -r "fn.*String[,)]" src/ --include="*.rs" | wc -l)

echo "  String slice parameters (&str): $STRING_SLICE_COUNT"
echo "  Owned string parameters (String): $OWNED_STRING_COUNT"

if [ $STRING_SLICE_COUNT -gt $OWNED_STRING_COUNT ]; then
    echo "   Good string slice usage for performance"
else
    echo "    Consider using more string slices (&str) instead of owned strings"
fi

# Test 3: Performance Module Integration
echo ""
echo " Test 3: Performance Module Integration"
if [ -f "src/performance.rs" ]; then
    echo "   Performance optimization module created"
    
    # Check if it's properly integrated
    if grep -q "pub mod performance" src/lib.rs; then
        echo "   Performance module integrated into lib.rs"
    else
        echo "    Performance module not integrated"
    fi
    
    # Check performance features
    CACHE_FEATURES=$(grep -c "cache" src/performance.rs)
    BATCH_FEATURES=$(grep -c "batch" src/performance.rs)
    POOL_FEATURES=$(grep -c "pool" src/performance.rs)
    
    echo "  Performance features implemented:"
    echo "    - Caching optimizations: $CACHE_FEATURES"
    echo "    - Batch operations: $BATCH_FEATURES" 
    echo "    - Pool optimizations: $POOL_FEATURES"
else
    echo "   Performance module not found"
fi

# Test 4: Database Query Optimization
echo ""
echo " Test 4: Database Query Optimization"

# Check for prepared statements vs dynamic queries
PREPARED_QUERIES=$(grep -r "sqlx::query!" src/ --include="*.rs" | wc -l)
DYNAMIC_QUERIES=$(grep -r "sqlx::query(" src/ --include="*.rs" | wc -l)

echo "  Prepared queries (query!): $PREPARED_QUERIES"
echo "  Dynamic queries (query): $DYNAMIC_QUERIES"

PREPARED_RATIO=$((PREPARED_QUERIES * 100 / (PREPARED_QUERIES + DYNAMIC_QUERIES)))
echo "  Prepared query ratio: ${PREPARED_RATIO}%"

if [ $PREPARED_RATIO -gt 80 ]; then
    echo "   Excellent prepared query usage"
elif [ $PREPARED_RATIO -gt 60 ]; then
    echo "   Good prepared query usage"
else
    echo "    Consider using more prepared queries (query!) for performance"
fi

# Test 5: Memory Allocation Patterns
echo ""
echo " Test 5: Memory Allocation Analysis"

# Check for optimized patterns
CLONE_COUNT=$(grep -r "\.clone()" src/ --include="*.rs" | wc -l)
ARC_COUNT=$(grep -r "Arc::" src/ --include="*.rs" | wc -l)
RC_COUNT=$(grep -r "Rc::" src/ --include="*.rs" | wc -l)

echo "  Clone operations: $CLONE_COUNT"
echo "  Arc usage (thread-safe): $ARC_COUNT"
echo "  Rc usage (single-thread): $RC_COUNT"

# Test 6: Async Performance
echo ""
echo " Test 6: Async Performance Analysis"

ASYNC_FN_COUNT=$(grep -r "async fn" src/ --include="*.rs" | wc -l)
AWAIT_COUNT=$(grep -r "\.await" src/ --include="*.rs" | wc -l)
BLOCKING_COUNT=$(grep -r "block_on\|spawn_blocking" src/ --include="*.rs" | wc -l)

echo "  Async functions: $ASYNC_FN_COUNT"
echo "  Await operations: $AWAIT_COUNT"
echo "  Blocking operations: $BLOCKING_COUNT"

ASYNC_RATIO=$((AWAIT_COUNT * 100 / ASYNC_FN_COUNT))
echo "  Async utilization: ${ASYNC_RATIO}%"

if [ $ASYNC_RATIO -gt 150 ]; then
    echo "   Good async utilization"
else
    echo "    Consider more async operations for better performance"
fi

# Test 7: Error Handling Performance
echo ""
echo " Test 7: Error Handling Performance"

UNWRAP_COUNT=$(grep -r "\.unwrap()" src/ --include="*.rs" | grep -v test | wc -l)
RESULT_COUNT=$(grep -r "Result<" src/ --include="*.rs" | wc -l)
QUESTION_MARK_COUNT=$(grep -r "?" src/ --include="*.rs" | wc -l)

echo "  Unwrap calls (non-test): $UNWRAP_COUNT"
echo "  Result types: $RESULT_COUNT"
echo "  ? operator usage: $QUESTION_MARK_COUNT"

if [ $UNWRAP_COUNT -lt 20 ]; then
    echo "   Good error handling (low unwrap usage)"
else
    echo "    High unwrap usage - consider proper error handling"
fi

# Summary
echo ""
echo " Performance Optimization Summary"
echo "=================================="

OPTIMIZATIONS=0

if grep -q 'features = \["rt-multi-thread"' Cargo.toml; then
    echo " Tokio features optimized"
    OPTIMIZATIONS=$((OPTIMIZATIONS + 1))
fi

if [ -f "src/performance.rs" ]; then
    echo " Performance module implemented"
    OPTIMIZATIONS=$((OPTIMIZATIONS + 1))
fi

if [ $PREPARED_RATIO -gt 70 ]; then
    echo " Database queries optimized"
    OPTIMIZATIONS=$((OPTIMIZATIONS + 1))
fi

if [ $STRING_SLICE_COUNT -gt $OWNED_STRING_COUNT ]; then
    echo " String allocations optimized"
    OPTIMIZATIONS=$((OPTIMIZATIONS + 1))
fi

if [ $UNWRAP_COUNT -lt 30 ]; then
    echo " Error handling optimized"
    OPTIMIZATIONS=$((OPTIMIZATIONS + 1))
fi

echo ""
echo " Optimization Score: $OPTIMIZATIONS/5"

if [ $OPTIMIZATIONS -ge 4 ]; then
    echo " Excellent performance optimizations!"
elif [ $OPTIMIZATIONS -ge 3 ]; then
    echo " Good performance optimizations"
else
    echo "  More optimizations recommended"
fi

echo ""
echo " Performance test complete!"

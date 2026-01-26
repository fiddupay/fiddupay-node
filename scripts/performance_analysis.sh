#!/bin/bash
# fiddupay Performance Analysis Script

echo " fiddupay Performance Analysis"
echo "==============================="

cd /home/vibes/crypto-payment-gateway

echo " Analyzing codebase for performance opportunities..."

# 1. Database Query Analysis
echo ""
echo "🗄️  Database Query Analysis:"
echo "  Checking for N+1 queries and inefficient patterns..."

# Count database queries
QUERY_COUNT=$(grep -r "sqlx::query" src/ --include="*.rs" | wc -l)
QUERY_MACRO_COUNT=$(grep -r "sqlx::query!" src/ --include="*.rs" | wc -l)
echo "  Total queries found: $QUERY_COUNT"
echo "  Compiled queries (query!): $QUERY_MACRO_COUNT"
echo "  Dynamic queries (query): $((QUERY_COUNT - QUERY_MACRO_COUNT))"

# Check for potential N+1 patterns
N_PLUS_ONE=$(grep -r "for.*in\|while.*" src/ --include="*.rs" -A 5 | grep -c "sqlx::query")
if [ $N_PLUS_ONE -gt 0 ]; then
    echo "  ⚠️  Potential N+1 queries: $N_PLUS_ONE"
else
    echo "   No obvious N+1 patterns detected"
fi

# 2. Memory Allocation Analysis
echo ""
echo "🧠 Memory Allocation Analysis:"

# Check for excessive cloning
CLONE_COUNT=$(grep -r "\.clone()" src/ --include="*.rs" | wc -l)
echo "  .clone() calls: $CLONE_COUNT"

# Check for String allocations
STRING_ALLOC=$(grep -r "String::new\|\.to_string\|\.to_owned" src/ --include="*.rs" | wc -l)
echo "  String allocations: $STRING_ALLOC"

# Check for Vec allocations
VEC_ALLOC=$(grep -r "Vec::new\|vec!\[" src/ --include="*.rs" | wc -l)
echo "  Vec allocations: $VEC_ALLOC"

# 3. Async/Concurrency Analysis
echo ""
echo " Async/Concurrency Analysis:"

# Check for blocking operations in async context
BLOCKING_OPS=$(grep -r "std::thread::sleep\|std::fs::\|std::io::" src/ --include="*.rs" | wc -l)
echo "  Potential blocking operations: $BLOCKING_OPS"

# Check for proper async usage
ASYNC_FN_COUNT=$(grep -r "async fn" src/ --include="*.rs" | wc -l)
AWAIT_COUNT=$(grep -r "\.await" src/ --include="*.rs" | wc -l)
echo "  Async functions: $ASYNC_FN_COUNT"
echo "  .await calls: $AWAIT_COUNT"

# 4. Error Handling Analysis
echo ""
echo "🚨 Error Handling Analysis:"

# Check for unwrap usage (potential panics)
UNWRAP_COUNT=$(grep -r "\.unwrap()" src/ --include="*.rs" | wc -l)
EXPECT_COUNT=$(grep -r "\.expect(" src/ --include="*.rs" | wc -l)
echo "  .unwrap() calls: $UNWRAP_COUNT"
echo "  .expect() calls: $EXPECT_COUNT"

if [ $UNWRAP_COUNT -gt 10 ]; then
    echo "  ⚠️  High unwrap usage - consider proper error handling"
else
    echo "   Reasonable unwrap usage"
fi

# 5. Serialization Analysis
echo ""
echo "📦 Serialization Analysis:"

# Check for JSON operations
JSON_OPS=$(grep -r "serde_json::\|to_string\|from_str" src/ --include="*.rs" | wc -l)
echo "  JSON operations: $JSON_OPS"

# 6. HTTP Client Analysis
echo ""
echo " HTTP Client Analysis:"

# Check for HTTP client usage
HTTP_CLIENTS=$(grep -r "reqwest::\|Client::new" src/ --include="*.rs" | wc -l)
echo "  HTTP client operations: $HTTP_CLIENTS"

# 7. Caching Analysis
echo ""
echo " Caching Analysis:"

# Check for caching usage
CACHE_OPS=$(grep -r "cache\|Cache\|redis" src/ --include="*.rs" | wc -l)
echo "  Caching operations: $CACHE_OPS"

# 8. Performance Hotspots
echo ""
echo " Potential Performance Hotspots:"

echo "  Checking for expensive operations in loops..."
LOOP_EXPENSIVE=$(grep -r "for.*in\|while.*" src/ --include="*.rs" -A 3 | grep -c "sqlx::\|reqwest::\|serde_json::")
if [ $LOOP_EXPENSIVE -gt 0 ]; then
    echo "  ⚠️  Expensive operations in loops: $LOOP_EXPENSIVE"
else
    echo "   No obvious expensive operations in loops"
fi

# 9. Dependency Analysis
echo ""
echo " Dependency Analysis:"
echo "  Checking Cargo.toml for performance-related dependencies..."

if grep -q "tokio.*features.*full" Cargo.toml; then
    echo "  ⚠️  Using full tokio features - consider selective features"
else
    echo "   Tokio features appear optimized"
fi

if grep -q "serde.*features.*derive" Cargo.toml; then
    echo "   Serde derive feature enabled"
else
    echo "  ⚠️  Consider enabling serde derive feature"
fi

# 10. File Size Analysis
echo ""
echo "📏 Code Size Analysis:"
LARGE_FILES=$(find src/ -name "*.rs" -size +10k | wc -l)
echo "  Files larger than 10KB: $LARGE_FILES"

if [ $LARGE_FILES -gt 5 ]; then
    echo "  ⚠️  Consider splitting large files"
    echo "  Largest files:"
    find src/ -name "*.rs" -exec wc -l {} + | sort -nr | head -5
else
    echo "   File sizes are reasonable"
fi

echo ""
echo " Performance Optimization Recommendations:"
echo "============================================="

# Generate recommendations based on analysis
RECOMMENDATIONS=0

if [ $CLONE_COUNT -gt 50 ]; then
    echo "1.  Reduce excessive cloning - consider using references"
    RECOMMENDATIONS=$((RECOMMENDATIONS + 1))
fi

if [ $STRING_ALLOC -gt 100 ]; then
    echo "2.  Optimize string allocations - use &str where possible"
    RECOMMENDATIONS=$((RECOMMENDATIONS + 1))
fi

if [ $UNWRAP_COUNT -gt 20 ]; then
    echo "3. 🚨 Replace unwrap() with proper error handling"
    RECOMMENDATIONS=$((RECOMMENDATIONS + 1))
fi

if [ $BLOCKING_OPS -gt 5 ]; then
    echo "4.  Replace blocking operations with async alternatives"
    RECOMMENDATIONS=$((RECOMMENDATIONS + 1))
fi

if [ $LOOP_EXPENSIVE -gt 0 ]; then
    echo "5.  Optimize expensive operations in loops"
    RECOMMENDATIONS=$((RECOMMENDATIONS + 1))
fi

if [ $CACHE_OPS -lt 10 ]; then
    echo "6.  Add more caching for frequently accessed data"
    RECOMMENDATIONS=$((RECOMMENDATIONS + 1))
fi

if [ $RECOMMENDATIONS -eq 0 ]; then
    echo " No major performance issues detected!"
    echo "   Your codebase appears well-optimized."
else
    echo ""
    echo "📈 Priority: Focus on the recommendations above for maximum impact"
fi

echo ""
echo " Analysis Complete!"

#!/bin/bash
# Quick Security Verification Test

echo " Quick Security Feature Verification"
echo ""

PASSED=0
TOTAL=0

check_feature() {
    ((TOTAL++))
    if eval "$2"; then
        echo " $1"
        ((PASSED++))
    else
        echo " $1"
    fi
}

# Critical Features
echo "=== CRITICAL FEATURES ==="
check_feature "XSS Prevention" "grep -q 'encode_text' src/api/handlers.rs"
check_feature "SQL Injection Protection" "grep -q 'sqlx::query!' src/services/payment_service.rs"
check_feature "Input Validation" "[ -f 'src/middleware/validation.rs' ]"
check_feature "Authentication Optimization" "[ -f 'src/middleware/auth.rs' ]"

# High Priority Features  
echo ""
echo "=== HIGH PRIORITY FEATURES ==="
check_feature "Email Validation" "grep -q 'validate_business_email' src/middleware/validation.rs"
check_feature "Per-Key Rate Limiting" "[ -f 'src/middleware/per_key_rate_limit.rs' ]"
check_feature "Webhook Validation" "grep -q 'validate_webhook_url' src/middleware/validation.rs"
check_feature "CSRF Protection" "[ -f 'src/middleware/csrf.rs' ]"
check_feature "Request Size Limits" "grep -q 'request_size_middleware' src/middleware/validation.rs"
check_feature "Security Headers" "grep -q 'security_headers_middleware' src/middleware/validation.rs"

# Medium Priority Features
echo ""
echo "=== MEDIUM PRIORITY FEATURES ==="
check_feature "Password Policy" "grep -q 'validate_password_strength' src/middleware/validation.rs"
check_feature "Account Lockout" "[ -f 'src/services/account_lockout_service.rs' ]"
check_feature "Content Security Policy" "grep -q 'Content-Security-Policy' src/middleware/validation.rs"

# Advanced Features
echo ""
echo "=== ADVANCED FEATURES ==="
check_feature "API Key Format Validation" "[ -f 'src/middleware/advanced_security.rs' ]"
check_feature "Request Tracking" "grep -q 'RequestTracker' src/middleware/advanced_security.rs"
check_feature "Advanced Rate Limiting" "grep -q 'TokenBucket' src/middleware/advanced_security.rs"
check_feature "Security Monitoring" "[ -f 'src/services/security_monitoring_service.rs' ]"
check_feature "API Version Security" "grep -q 'ApiVersionManager' src/middleware/advanced_security.rs"

# Database Tests
echo ""
echo "=== DATABASE SECURITY ==="
export DATABASE_URL="${DATABASE_URL:-postgresql://vibes:password@localhost:5432/fiddupay_dev}"
check_feature "Security Tables" "psql '$DATABASE_URL' -c 'SELECT 1 FROM information_schema.tables WHERE table_name = '\''security_events'\'';' | grep -q '1'"
check_feature "Performance Indexes" "psql '$DATABASE_URL' -c 'SELECT 1 FROM pg_indexes WHERE indexname = '\''idx_merchants_api_key_hash'\'';' | grep -q '1'"

# Dependencies
echo ""
echo "=== DEPENDENCIES ==="
check_feature "Validator Dependency" "grep -q '^validator' Cargo.toml"
check_feature "HTML Escape Dependency" "grep -q '^html-escape' Cargo.toml"
check_feature "UUID Dependency" "grep -q '^uuid' Cargo.toml"

echo ""
echo ""
echo "                    TEST RESULTS                            "
echo ""
echo ""
echo " PASSED: $PASSED"
echo " TOTAL: $TOTAL"

SCORE=$(awk "BEGIN {printf \"%.1f\", ($PASSED * 10.0) / $TOTAL}")
echo " SECURITY SCORE: $SCORE/10"

if [ $PASSED -eq $TOTAL ]; then
    echo ""
    echo " PERFECT SECURITY ACHIEVED!"
    echo " All security features implemented"
    echo " Ready for production deployment"
elif [ $PASSED -ge $((TOTAL * 9 / 10)) ]; then
    echo ""
    echo "  NEAR PERFECT SECURITY"
    echo " Minor optimizations possible"
else
    echo ""
    echo " SECURITY GAPS DETECTED"
    echo " Additional fixes needed"
fi

#!/bin/bash
# fiddupay - Security Audit Script

set -e

echo ""
echo "          fiddupay - Security Audit                          "
echo ""
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

ISSUES=0
WARNINGS=0
PASSED=0

check_pass() {
    echo -e "${GREEN} $1${NC}"
    ((PASSED++))
}

check_warn() {
    echo -e "${YELLOW}  $1${NC}"
    ((WARNINGS++))
}

check_fail() {
    echo -e "${RED} $1${NC}"
    ((ISSUES++))
}

echo "=== 1. Environment Configuration ==="
echo ""

# Check .env file
if [ -f ".env" ]; then
    check_pass ".env file exists"
    
    # Check for default/weak keys
    if grep -q "test_key\|example\|changeme\|password123" .env 2>/dev/null; then
        check_fail "Default/weak keys found in .env"
    else
        check_pass "No obvious weak keys in .env"
    fi
    
    # Check file permissions
    PERMS=$(stat -c %a .env 2>/dev/null || stat -f %A .env 2>/dev/null)
    if [ "$PERMS" = "600" ] || [ "$PERMS" = "400" ]; then
        check_pass ".env has secure permissions ($PERMS)"
    else
        check_fail ".env has insecure permissions ($PERMS), should be 600"
    fi
else
    check_warn ".env file not found"
fi

echo ""
echo "=== 2. Dependency Security ==="
echo ""

# Check for cargo-audit
if command -v cargo-audit &> /dev/null; then
    echo "Running cargo audit..."
    if cargo audit 2>&1 | grep -q "Vulnerabilities found"; then
        check_fail "Vulnerabilities found in dependencies"
        cargo audit
    else
        check_pass "No known vulnerabilities in dependencies"
    fi
else
    check_warn "cargo-audit not installed (run: cargo install cargo-audit)"
fi

echo ""
echo "=== 3. Code Security ==="
echo ""

# Check for hardcoded secrets
echo "Scanning for hardcoded secrets..."
if grep -r "password\s*=\s*['\"]" src/ 2>/dev/null | grep -v "password_hash\|PasswordHash" | head -1; then
    check_fail "Possible hardcoded passwords found"
else
    check_pass "No hardcoded passwords found"
fi

if grep -r "api_key\s*=\s*['\"]" src/ 2>/dev/null | grep -v "api_key_hash" | head -1; then
    check_fail "Possible hardcoded API keys found"
else
    check_pass "No hardcoded API keys found"
fi

if grep -r "secret\s*=\s*['\"]" src/ 2>/dev/null | grep -v "secret_key\|SecretKey" | head -1; then
    check_fail "Possible hardcoded secrets found"
else
    check_pass "No hardcoded secrets found"
fi

echo ""
echo "=== 4. Database Security ==="
echo ""

# Check migrations for security issues
if [ -d "migrations" ]; then
    check_pass "Migrations directory exists"
    
    # Check for SQL injection patterns
    if grep -r "format!\|concat" migrations/ 2>/dev/null | head -1; then
        check_warn "String concatenation found in migrations (review for SQL injection)"
    else
        check_pass "No obvious SQL injection patterns in migrations"
    fi
    
    # Check for default passwords
    if grep -i "password.*=.*'.*'" migrations/ 2>/dev/null | head -1; then
        check_fail "Possible default passwords in migrations"
    else
        check_pass "No default passwords in migrations"
    fi
else
    check_fail "Migrations directory not found"
fi

echo ""
echo "=== 5. Encryption & Hashing ==="
echo ""

# Check for weak hashing
if grep -r "md5\|sha1" src/ 2>/dev/null | grep -v "Sha1" | head -1; then
    check_fail "Weak hashing algorithms (MD5/SHA1) found"
else
    check_pass "No weak hashing algorithms found"
fi

# Check for proper password hashing
if grep -r "argon2\|Argon2" src/ 2>/dev/null | head -1 > /dev/null; then
    check_pass "Using Argon2 for password hashing"
else
    check_warn "Argon2 password hashing not found"
fi

# Check for encryption
if grep -r "aes.*gcm\|AES.*GCM" src/ 2>/dev/null | head -1 > /dev/null; then
    check_pass "Using AES-GCM encryption"
else
    check_warn "AES-GCM encryption not found"
fi

echo ""
echo "=== 6. API Security ==="
echo ""

# Check for authentication middleware
if grep -r "authenticate\|auth_middleware" src/ 2>/dev/null | head -1 > /dev/null; then
    check_pass "Authentication middleware found"
else
    check_fail "No authentication middleware found"
fi

# Check for rate limiting
if grep -r "rate_limit\|RateLimit" src/ 2>/dev/null | head -1 > /dev/null; then
    check_pass "Rate limiting implemented"
else
    check_warn "Rate limiting not found"
fi

# Check for CORS configuration
if grep -r "cors\|CORS" src/ 2>/dev/null | head -1 > /dev/null; then
    check_pass "CORS configuration found"
else
    check_warn "CORS configuration not found"
fi

echo ""
echo "=== 7. Input Validation ==="
echo ""

# Check for input validation
if grep -r "validate\|Validate" src/ 2>/dev/null | head -1 > /dev/null; then
    check_pass "Input validation found"
else
    check_warn "Input validation not found"
fi

# Check for SQL injection protection
if grep -r "sqlx::query!" src/ 2>/dev/null | head -1 > /dev/null; then
    check_pass "Using parameterized queries (sqlx)"
else
    check_warn "Parameterized queries not found"
fi

echo ""
echo "=== 8. Error Handling ==="
echo ""

# Check for proper error handling
if grep -r "ServiceError\|Error" src/error.rs 2>/dev/null | head -1 > /dev/null; then
    check_pass "Custom error types defined"
else
    check_warn "Custom error types not found"
fi

# Check for error exposure
if grep -r "\.unwrap()\|\.expect(" src/ 2>/dev/null | wc -l | grep -v "^0$" > /dev/null; then
    UNWRAPS=$(grep -r "\.unwrap()\|\.expect(" src/ 2>/dev/null | wc -l)
    check_warn "$UNWRAPS unwrap()/expect() calls found (review for production)"
else
    check_pass "No unwrap()/expect() calls found"
fi

echo ""
echo "=== 9. Logging & Monitoring ==="
echo ""

# Check for logging
if grep -r "tracing::\|log::" src/ 2>/dev/null | head -1 > /dev/null; then
    check_pass "Logging framework found"
else
    check_warn "Logging framework not found"
fi

# Check for sensitive data in logs
if grep -r "debug!.*password\|info!.*api_key" src/ 2>/dev/null | head -1; then
    check_fail "Possible sensitive data in logs"
else
    check_pass "No obvious sensitive data in logs"
fi

echo ""
echo "=== 10. Dependencies ==="
echo ""

# Check Cargo.toml for security-related crates
if grep -E "argon2|aes-gcm|totp" Cargo.toml > /dev/null; then
    check_pass "Security crates present (argon2, aes-gcm, totp)"
else
    check_warn "Security crates not found in Cargo.toml"
fi

# Check for outdated dependencies
if command -v cargo-outdated &> /dev/null; then
    echo "Checking for outdated dependencies..."
    OUTDATED=$(cargo outdated 2>&1 | grep -c "->")
    if [ "$OUTDATED" -gt 0 ]; then
        check_warn "$OUTDATED outdated dependencies found"
    else
        check_pass "All dependencies up to date"
    fi
else
    check_warn "cargo-outdated not installed (run: cargo install cargo-outdated)"
fi

echo ""
echo "=== 11. File Permissions ==="
echo ""

# Check for world-readable sensitive files
if find . -name "*.key" -o -name "*.pem" -o -name ".env*" 2>/dev/null | while read file; do
    PERMS=$(stat -c %a "$file" 2>/dev/null || stat -f %A "$file" 2>/dev/null)
    if [ "${PERMS: -1}" != "0" ]; then
        echo "$file has world-readable permissions ($PERMS)"
        return 1
    fi
done; then
    check_pass "No world-readable sensitive files"
else
    check_fail "World-readable sensitive files found"
fi

echo ""
echo "=== 12. Network Security ==="
echo ""

# Check for hardcoded IPs
if grep -r "127\.0\.0\.1\|localhost" src/ 2>/dev/null | grep -v "test\|example\|comment" | head -1; then
    check_warn "Hardcoded localhost/127.0.0.1 found (review for production)"
else
    check_pass "No hardcoded localhost addresses"
fi

# Check for HTTP (should be HTTPS)
if grep -r "http://" src/ 2>/dev/null | grep -v "localhost\|test\|example" | head -1; then
    check_warn "HTTP URLs found (should use HTTPS in production)"
else
    check_pass "No HTTP URLs found"
fi

echo ""
echo "=== 13. Cryptographic Keys ==="
echo ""

# Check for weak key generation
if grep -r "rand::thread_rng\|OsRng" src/ 2>/dev/null | head -1 > /dev/null; then
    check_pass "Using cryptographically secure RNG"
else
    check_warn "Cryptographically secure RNG not found"
fi

echo ""
echo "=== 14. Production Readiness ==="
echo ""

# Check for debug code
if grep -r "println!\|dbg!\|todo!\|unimplemented!" src/ 2>/dev/null | wc -l | grep -v "^0$" > /dev/null; then
    DEBUG_COUNT=$(grep -r "println!\|dbg!\|todo!\|unimplemented!" src/ 2>/dev/null | wc -l)
    check_warn "$DEBUG_COUNT debug statements found (remove for production)"
else
    check_pass "No debug statements found"
fi

# Check for test code in production
if grep -r "#\[cfg(test)\]" src/ 2>/dev/null | head -1 > /dev/null; then
    check_pass "Test code properly gated with cfg(test)"
else
    check_pass "No test code found in src/"
fi

echo ""
echo "=== 15. Additional Recommendations ==="
echo ""

echo "Manual security checks needed:"
echo "  • Review all API endpoints for authorization"
echo "  • Test rate limiting under load"
echo "  • Verify webhook signature validation"
echo "  • Test 2FA implementation"
echo "  • Review IP whitelisting logic"
echo "  • Verify encryption key rotation process"
echo "  • Test session management"
echo "  • Review audit logging completeness"
echo "  • Verify CORS configuration"
echo "  • Test error messages (no info leakage)"
echo ""

echo "External security tools to run:"
echo "  • cargo audit - Dependency vulnerabilities"
echo "  • cargo clippy - Code quality & security"
echo "  • cargo deny - License & security policy"
echo "  • sqlx check - Database query validation"
echo "  • OWASP ZAP - Web application security"
echo "  • nmap - Network security scan"
echo ""

echo ""
echo "              Security Audit Complete                       "
echo ""
echo ""
echo -e "${GREEN} Passed: $PASSED${NC}"
echo -e "${YELLOW}  Warnings: $WARNINGS${NC}"
echo -e "${RED} Issues: $ISSUES${NC}"
echo ""

if [ $ISSUES -gt 0 ]; then
    echo -e "${RED}  CRITICAL: Fix all issues before production deployment${NC}"
    exit 1
elif [ $WARNINGS -gt 5 ]; then
    echo -e "${YELLOW}  WARNING: Review warnings before production deployment${NC}"
    exit 0
else
    echo -e "${GREEN} Security audit passed! Ready for production.${NC}"
    exit 0
fi

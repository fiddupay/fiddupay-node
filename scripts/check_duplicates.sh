#!/bin/bash
# Duplicate Environment Variable Detection Script

echo "🔍 PayFlow Duplicate Variable Detection"
echo "======================================"

cd /home/vibes/crypto-payment-gateway

# Check .env file for duplicates
echo "📋 Checking .env file for duplicates..."
if [ -f .env ]; then
    DUPLICATES=$(cut -d'=' -f1 .env | grep -v '^#' | grep -v '^$' | sort | uniq -d)
    if [ -n "$DUPLICATES" ]; then
        echo "❌ Found duplicate variables in .env:"
        echo "$DUPLICATES"
        echo ""
        echo "Details:"
        for var in $DUPLICATES; do
            echo "  $var appears in these lines:"
            grep -n "^$var=" .env
        done
    else
        echo "✅ No duplicates found in .env"
    fi
else
    echo "❌ .env file not found"
fi

echo ""

# Check .env.example file for duplicates
echo "📋 Checking .env.example file for duplicates..."
if [ -f .env.example ]; then
    DUPLICATES_EXAMPLE=$(cut -d'=' -f1 .env.example | grep -v '^#' | grep -v '^$' | sort | uniq -d)
    if [ -n "$DUPLICATES_EXAMPLE" ]; then
        echo "❌ Found duplicate variables in .env.example:"
        echo "$DUPLICATES_EXAMPLE"
        echo ""
        echo "Details:"
        for var in $DUPLICATES_EXAMPLE; do
            echo "  $var appears in these lines:"
            grep -n "^$var=" .env.example
        done
    else
        echo "✅ No duplicates found in .env.example"
    fi
else
    echo "❌ .env.example file not found"
fi

echo ""

# Check for variables in .env.example but not in .env
echo "📋 Checking for missing variables in .env..."
if [ -f .env ] && [ -f .env.example ]; then
    ENV_VARS=$(cut -d'=' -f1 .env | grep -v '^#' | grep -v '^$' | sort)
    EXAMPLE_VARS=$(cut -d'=' -f1 .env.example | grep -v '^#' | grep -v '^$' | sort)
    
    MISSING_IN_ENV=$(comm -23 <(echo "$EXAMPLE_VARS") <(echo "$ENV_VARS"))
    if [ -n "$MISSING_IN_ENV" ]; then
        echo "⚠️  Variables in .env.example but missing in .env:"
        echo "$MISSING_IN_ENV"
    else
        echo "✅ All .env.example variables are present in .env"
    fi
    
    echo ""
    
    EXTRA_IN_ENV=$(comm -13 <(echo "$EXAMPLE_VARS") <(echo "$ENV_VARS"))
    if [ -n "$EXTRA_IN_ENV" ]; then
        echo "ℹ️  Extra variables in .env (not in .env.example):"
        echo "$EXTRA_IN_ENV"
    else
        echo "✅ No extra variables in .env"
    fi
fi

echo ""

# Check for duplicate env::var calls in Rust code
echo "📋 Checking Rust code for duplicate env::var calls..."
RUST_ENV_VARS=$(grep -r "env::var(" src/ --include="*.rs" | sed 's/.*env::var("\([^"]*\)").*/\1/' | sort | uniq -c | sort -nr)

echo "Environment variables used in Rust code (with frequency):"
echo "$RUST_ENV_VARS" | head -20

RUST_DUPLICATES=$(echo "$RUST_ENV_VARS" | awk '$1 > 5 {print $2}')
if [ -n "$RUST_DUPLICATES" ]; then
    echo ""
    echo "⚠️  Variables used more than 5 times (potential over-usage):"
    echo "$RUST_DUPLICATES"
fi

echo ""
echo "🎯 Summary:"
ENV_COUNT=$(grep -c "^[A-Z]" .env 2>/dev/null || echo "0")
EXAMPLE_COUNT=$(grep -c "^[A-Z]" .env.example 2>/dev/null || echo "0")
echo "  .env variables: $ENV_COUNT"
echo "  .env.example variables: $EXAMPLE_COUNT"

if [ -z "$DUPLICATES" ] && [ -z "$DUPLICATES_EXAMPLE" ]; then
    echo "✅ No duplicate variables found in environment files"
else
    echo "❌ Duplicate variables found - needs cleanup"
fi

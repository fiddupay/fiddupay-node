#!/bin/bash
# Test Files Analysis and Cleanup

echo " Analyzing test files..."

# Count files by category
RUST_TESTS=($(find tests -name "*.rs" | wc -l))
SHELL_TESTS=($(find tests -name "*.sh" | wc -l))

echo "Found:"
echo "- $RUST_TESTS Rust test files"
echo "- $SHELL_TESTS Shell test scripts"
echo ""

echo " Rust Test Files Analysis:"
for file in tests/*/*.rs; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        tests=$(grep -c "#\[test\]" "$file" 2>/dev/null || echo "0")
        echo "  $(basename "$file") - $lines lines, $tests tests"
    fi
done

echo ""
echo " Shell Test Scripts Analysis:"
for file in tests/scripts/*.sh; do
    if [ -f "$file" ]; then
        lines=$(wc -l < "$file")
        functions=$(grep -c "^test_" "$file" 2>/dev/null || echo "0")
        echo "  $(basename "$file") - $lines lines, $functions test functions"
    fi
done

echo ""
echo " Identifying duplicates and outdated tests..."

# Check for similar test content
echo "Potential duplicates:"
echo "- payment_test.rs vs payment_listing_tests.rs (payment functionality)"
echo "- services_test.rs vs comprehensive_service_test.rs (service testing)"
echo "- test_complete_flow.sh vs test_complete_workflows.sh (workflow testing)"
echo "- test.sh (generic) vs specific test scripts"

echo ""
echo " Cleanup recommendations:"
echo "1. Merge payment_test.rs into payment_listing_tests.rs"
echo "2. Keep comprehensive_service_test.rs, remove services_test.rs"
echo "3. Keep test_complete_workflows.sh, remove test_complete_flow.sh"
echo "4. Remove generic test.sh"
echo "5. Consolidate similar API tests"

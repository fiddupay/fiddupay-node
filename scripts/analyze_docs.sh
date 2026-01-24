#!/bin/bash
# Documentation Cleanup and Consolidation Script

echo "📚 Analyzing documentation files..."

# Categorize files
SECURITY_FILES=(
    "SECURITY_AUDIT_REPORT.md"
    "SECURITY_AUDIT_REPORT_UPDATED.md" 
    "SECURITY_COMPLETE.md"
    "PERFECT_SECURITY_ACHIEVED.md"
    "FINAL_SECURITY_STATUS.md"
    "SECURITY_IMPLEMENTATION_COMPLETE.md"
    "COMPREHENSIVE_TEST_RESULTS.md"
    "REMAINING_SECURITY_GAPS.md"
    "README_SECURITY.md"
    "SECURITY_FIXES_GUIDE.md"
    "docs/GIT_SECURITY.md"
)

DOCS_FILES=(
    "docs/API_REFERENCE.md"
    "docs/API.md"
    "docs/DEPLOYMENT.md"
    "docs/DOCUMENTATION_INDEX.md"
    "docs/MERCHANT_GUIDE.md"
    "docs/PRICE_API_REFERENCE.md"
    "docs/PROJECT_STATUS.md"
    "docs/PROJECT_STRUCTURE.md"
    "docs/ROADMAP.md"
    "docs/SETUP.md"
    "docs/TESTING.md"
)

MAIN_FILES=(
    "README.md"
    "tests/README.md"
)

echo "Found:"
echo "- ${#SECURITY_FILES[@]} security documentation files"
echo "- ${#DOCS_FILES[@]} docs/ directory files"
echo "- ${#MAIN_FILES[@]} main documentation files"
echo ""

# Check for duplicates and outdated content
echo "🔍 Checking for duplicates and outdated content..."

# Security files analysis
echo "Security files to consolidate:"
for file in "${SECURITY_FILES[@]}"; do
    if [ -f "$file" ]; then
        size=$(wc -l < "$file")
        echo "  $file ($size lines)"
    fi
done

echo ""
echo "📋 Consolidation plan:"
echo "1. Keep: SECURITY_AUDIT_REPORT_UPDATED.md (most comprehensive)"
echo "2. Keep: COMPREHENSIVE_TEST_RESULTS.md (test results)"
echo "3. Delete: All other security files (duplicates/outdated)"
echo "4. Merge: docs/API.md into docs/API_REFERENCE.md"
echo "5. Update: README.md with final status"

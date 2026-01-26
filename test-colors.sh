#!/bin/bash

echo "🎨 FIDDUPAY COLOR SYSTEM AUDIT"
echo "================================"
echo ""

echo "✅ TESTING COLOR CONSISTENCY ACROSS ALL PAGES"
echo ""

# Check for white text on white background issues
echo "🔍 Checking for white-on-white issues..."
if grep -r "color.*#ffffff" frontend/src/ | grep -q "background.*#ffffff"; then
    echo "❌ Found white text on white background!"
    grep -r "color.*#ffffff" frontend/src/ | grep "background.*#ffffff"
else
    echo "✅ No white-on-white issues found"
fi

echo ""

# Check for consistent background usage
echo "🔍 Checking background color consistency..."
echo "Pages using white background:"
grep -r "background.*var(--white)" frontend/src/ | wc -l
echo "Pages using other backgrounds:"
grep -r "background.*#" frontend/src/ | grep -v "var(--white)" | grep -v "var(--brand" | wc -l

echo ""

# Check for consistent text colors
echo "🔍 Checking text color consistency..."
echo "Elements using black text:"
grep -r "color.*var(--black)" frontend/src/ | wc -l
echo "Elements using gray text:"
grep -r "color.*var(--gray)" frontend/src/ | wc -l

echo ""

# Check for proper brand color usage
echo "🔍 Checking brand color usage..."
echo "Brand blue usage:"
grep -r "var(--brand-blue)" frontend/src/ | wc -l
echo "Brand green usage:"
grep -r "var(--brand-green)" frontend/src/ | wc -l
echo "Brand gold usage:"
grep -r "var(--brand-gold)" frontend/src/ | wc -l

echo ""

echo "🎯 COLOR SYSTEM RULES:"
echo "1. Background: Always white (var(--white)) or light gray (var(--light-gray))"
echo "2. Text: Always black (var(--black)) or gray (var(--gray))"
echo "3. Brand colors: Only for buttons, links, status badges"
echo "4. Never: Same color on same color"
echo ""

echo "📊 PAGES TO TEST:"
echo "- HomePage: /color-test (test page)"
echo "- PaymentsPage: /app/payments"
echo "- StatusPage: /status"
echo "- All other pages should follow same rules"
echo ""

echo "🚀 BUILD STATUS:"
if cd frontend && npm run build > /dev/null 2>&1; then
    echo "✅ Frontend builds successfully"
else
    echo "❌ Frontend build failed"
fi

echo ""
echo "✅ COLOR AUDIT COMPLETE"

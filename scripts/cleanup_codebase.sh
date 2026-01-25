#!/bin/bash

# FidduPay Codebase Cleanup Script
# Removes temporary files, logs, and unnecessary artifacts

set -e

echo "🧹 FidduPay Codebase Cleanup"
echo "============================"
echo ""

# Function to safely remove files/directories
safe_remove() {
    if [ -e "$1" ]; then
        echo "🗑️  Removing: $1"
        rm -rf "$1"
    fi
}

# Function to clean directory contents but keep directory
clean_directory() {
    if [ -d "$1" ]; then
        echo "🧽 Cleaning: $1/*"
        rm -rf "$1"/*
    fi
}

echo "1️⃣  Cleaning build artifacts..."
safe_remove "backend/target"
safe_remove "frontend/dist"
safe_remove "frontend/build"
safe_remove "fiddupay-node-sdk/dist"
safe_remove "fiddupay-node-sdk/coverage"

echo "2️⃣  Cleaning logs..."
safe_remove "backend/server.log"
safe_remove "backend/server_output.log"
clean_directory "logs"

echo "3️⃣  Cleaning temporary files..."
find . -name "*.tmp" -type f -delete 2>/dev/null || true
find . -name "*.temp" -type f -delete 2>/dev/null || true
find . -name ".DS_Store" -type f -delete 2>/dev/null || true
find . -name "Thumbs.db" -type f -delete 2>/dev/null || true

echo "4️⃣  Cleaning node_modules (keeping package-lock.json)..."
safe_remove "frontend/node_modules"
safe_remove "fiddupay-node-sdk/node_modules"
safe_remove "node_modules"

echo "5️⃣  Cleaning Rust artifacts..."
safe_remove "backend/Cargo.lock"
find backend -name "*.rlib" -type f -delete 2>/dev/null || true

echo "6️⃣  Cleaning IDE files..."
safe_remove ".vscode/settings.json"
safe_remove ".idea"
find . -name "*.swp" -type f -delete 2>/dev/null || true
find . -name "*.swo" -type f -delete 2>/dev/null || true

echo "7️⃣  Cleaning test artifacts..."
safe_remove "coverage"
safe_remove ".nyc_output"
safe_remove "test-results"

echo "8️⃣  Cleaning environment files..."
safe_remove ".env.local"
safe_remove ".env.development"
safe_remove ".env.test"
safe_remove "backend/.env.local"
safe_remove "frontend/.env.local"

echo "9️⃣  Cleaning cache files..."
safe_remove ".cache"
safe_remove "backend/.sqlx"
safe_remove "frontend/.vite"
safe_remove ".npm"

echo "🔟 Cleaning screenshots and images..."
safe_remove "image1.png"
safe_remove "image2.png" 
safe_remove "image3.png"

echo "1️⃣1️⃣ Cleaning duplicate/old files..."
safe_remove "frontend/src/pages/PricingPageOld.tsx"
safe_remove "frontend/src/pages/PricingPageOld.module.css"

echo "1️⃣2️⃣ Cleaning git artifacts..."
find . -name ".git" -type d -not -path "./.git" -exec rm -rf {} + 2>/dev/null || true

echo ""
echo "📊 Cleanup Summary:"
echo "=================="

# Show directory sizes
echo "Backend size: $(du -sh backend 2>/dev/null | cut -f1)"
echo "Frontend size: $(du -sh frontend 2>/dev/null | cut -f1)"
echo "SDK size: $(du -sh fiddupay-node-sdk 2>/dev/null | cut -f1)"
echo "Scripts size: $(du -sh scripts 2>/dev/null | cut -f1)"
echo "Docs size: $(du -sh docs 2>/dev/null | cut -f1)"

echo ""
echo "Total project size: $(du -sh . 2>/dev/null | cut -f1)"

echo ""
echo "✅ Codebase cleanup completed!"
echo ""
echo "🎯 What's preserved:"
echo "- Source code files"
echo "- Configuration files"
echo "- Package.json files"
echo "- Documentation"
echo "- Git history"
echo ""
echo "🗑️  What's removed:"
echo "- Build artifacts"
echo "- Log files"
echo "- Node modules"
echo "- Temporary files"
echo "- Cache files"
echo "- IDE artifacts"

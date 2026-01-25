#!/bin/bash

# Deep Codebase Cleanup - More aggressive cleanup

set -e

echo "🔥 Deep Codebase Cleanup"
echo "========================"
echo ""

read -p "⚠️  This will remove ALL build artifacts, logs, and caches. Continue? (y/N): " confirm

if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "❌ Cleanup cancelled."
    exit 1
fi

echo ""
echo "🧹 Starting deep cleanup..."

# Remove all build and cache directories
echo "1️⃣  Removing build artifacts..."
find . -name "target" -type d -exec rm -rf {} + 2>/dev/null || true
find . -name "dist" -type d -exec rm -rf {} + 2>/dev/null || true
find . -name "build" -type d -exec rm -rf {} + 2>/dev/null || true
find . -name "coverage" -type d -exec rm -rf {} + 2>/dev/null || true

echo "2️⃣  Removing node_modules..."
find . -name "node_modules" -type d -exec rm -rf {} + 2>/dev/null || true

echo "3️⃣  Removing logs and temporary files..."
find . -name "*.log" -type f -delete 2>/dev/null || true
find . -name "*.tmp" -type f -delete 2>/dev/null || true
find . -name "*.temp" -type f -delete 2>/dev/null || true

echo "4️⃣  Removing cache files..."
find . -name ".cache" -type d -exec rm -rf {} + 2>/dev/null || true
find . -name ".npm" -type d -exec rm -rf {} + 2>/dev/null || true
find . -name ".sqlx" -type d -exec rm -rf {} + 2>/dev/null || true

echo "5️⃣  Removing IDE and OS files..."
find . -name ".DS_Store" -type f -delete 2>/dev/null || true
find . -name "Thumbs.db" -type f -delete 2>/dev/null || true
find . -name "*.swp" -type f -delete 2>/dev/null || true
find . -name "*.swo" -type f -delete 2>/dev/null || true

echo "6️⃣  Removing duplicate git repositories..."
find . -name ".git" -type d -not -path "./.git" -exec rm -rf {} + 2>/dev/null || true

echo "7️⃣  Removing package tarballs..."
find . -name "*.tgz" -type f -delete 2>/dev/null || true

echo ""
echo "📊 Final project size: $(du -sh . 2>/dev/null | cut -f1)"
echo ""
echo "✅ Deep cleanup completed!"
echo "🎯 Your codebase is now clean and optimized!"

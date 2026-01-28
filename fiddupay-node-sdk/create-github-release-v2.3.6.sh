#!/bin/bash

# FidduPay Node.js SDK v2.3.6 GitHub Release Script
# This script creates a comprehensive GitHub release for the standalone SDK repository

set -e

echo "🚀 Creating FidduPay Node.js SDK v2.3.6 GitHub Release..."

# Configuration
REPO="fiddupay/fiddupay-node"
TAG="v2.3.6"
RELEASE_NAME="FidduPay Node.js SDK v2.3.6 - API Centralization Release"

# Check if GitHub CLI is available
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is not installed. Please install it first:"
    echo "   https://cli.github.com/"
    exit 1
fi

# Check if we're authenticated
if ! gh auth status &> /dev/null; then
    echo "❌ Not authenticated with GitHub CLI. Please run 'gh auth login' first."
    exit 1
fi

echo "✅ GitHub CLI is available and authenticated"

# Read the release notes from file
RELEASE_NOTES_FILE="GITHUB_RELEASE_NOTES_v2.3.6.md"

if [ ! -f "$RELEASE_NOTES_FILE" ]; then
    echo "❌ Release notes file not found: $RELEASE_NOTES_FILE"
    exit 1
fi

echo "✅ Release notes file found"

# Create the GitHub release
echo "📝 Creating GitHub release..."

gh release create "$TAG" \
    --repo "$REPO" \
    --title "$RELEASE_NAME" \
    --notes-file "$RELEASE_NOTES_FILE" \
    --latest \
    --verify-tag

if [ $? -eq 0 ]; then
    echo "🎉 Successfully created GitHub release v2.3.6!"
    echo "🔗 Release URL: https://github.com/$REPO/releases/tag/$TAG"
    
    # Add additional files to the release
    echo "📎 Adding additional files to the release..."
    
    # Add migration guide
    if [ -f "MIGRATION_GUIDE_v2.3.6.md" ]; then
        gh release upload "$TAG" "MIGRATION_GUIDE_v2.3.6.md" --repo "$REPO"
        echo "✅ Added migration guide to release"
    fi
    
    # Add changelog
    if [ -f "CHANGELOG.md" ]; then
        gh release upload "$TAG" "CHANGELOG.md" --repo "$REPO"
        echo "✅ Added changelog to release"
    fi
    
    # Add package.json for reference
    if [ -f "package.json" ]; then
        gh release upload "$TAG" "package.json" --repo "$REPO"
        echo "✅ Added package.json to release"
    fi
    
    echo ""
    echo "🎊 Release v2.3.6 is now live!"
    echo "📋 Release includes:"
    echo "   • Comprehensive release notes"
    echo "   • Migration guide"
    echo "   • Updated changelog"
    echo "   • Package configuration"
    echo ""
    echo "🔗 View the release: https://github.com/$REPO/releases/tag/$TAG"
    echo "📦 Install with: npm install @fiddupay/fiddupay-node@2.3.6"
    
else
    echo "❌ Failed to create GitHub release"
    exit 1
fi

echo "✨ GitHub release creation completed successfully!"
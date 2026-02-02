#!/bin/bash

# Automated Version Update Script
# Usage: ./update-version.sh [patch|minor|major]

set -e

VERSION_TYPE=${1:-patch}
CURRENT_DIR=$(pwd)

echo " FidduPay SDK Automated Version Update"
echo "========================================"

# Check if we're in the SDK directory
if [[ ! -f "package.json" ]] || [[ ! $(grep -q "@fiddupay/fiddupay-node" package.json) ]]; then
    echo " Error: Must be run from the SDK directory"
    exit 1
fi

# Get current version
CURRENT_VERSION=$(node -p "require('./package.json').version")
echo " Current version: $CURRENT_VERSION"

# Update version
echo "  Updating version ($VERSION_TYPE)..."
NEW_VERSION=$(npm version $VERSION_TYPE --no-git-tag-version)
NEW_VERSION=${NEW_VERSION#v}  # Remove 'v' prefix

echo " New version: $NEW_VERSION"

# Run tests
echo " Running tests..."
npm test

# Build package
echo " Building package..."
npm run build

# Update documentation files
echo " Updating documentation..."

# Update README badges (they auto-update, but we can trigger refresh)
echo "   - README badges will auto-update"

# Update any hardcoded version references in docs
find . -name "*.md" -not -path "./node_modules/*" -exec sed -i "s/v$CURRENT_VERSION/v$NEW_VERSION/g" {} \;
find . -name "*.md" -not -path "./node_modules/*" -exec sed -i "s/@$CURRENT_VERSION/@$NEW_VERSION/g" {} \;

# Commit changes
echo " Committing changes..."
git add .
git commit -m "release: bump version to $NEW_VERSION

 AUTOMATED VERSION UPDATE:
- Updated package.json: $CURRENT_VERSION → $NEW_VERSION
- Updated documentation references
- Tests passing: 
- Build successful: 
- Ready for release"

# Create and push tag
echo "  Creating release tag..."
git tag "v$NEW_VERSION"
git push origin main
git push origin "v$NEW_VERSION"

echo ""
echo " Version update complete!"
echo " New version: $NEW_VERSION"
echo "  Tag: v$NEW_VERSION"
echo ""
echo " Next steps:"
echo "   1. GitHub Actions will automatically:"
echo "      - Publish to NPM"
echo "      - Create GitHub release"
echo "      - Update all badges"
echo "   2. Check: https://github.com/fiddupay/fiddupay-node/releases"
echo "   3. Verify: https://www.npmjs.com/package/@fiddupay/fiddupay-node"

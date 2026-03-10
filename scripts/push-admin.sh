#!/bin/bash

# Script to push Admin Dashboard changes to the isolated Admin repository
# Usage: ./scripts/push-admin.sh [branch] [tag]
# bash scripts/push-admin.sh main

BRANCH=${1:-main}
TAG=$2
REMOTE_URL="https://github.com/fiddupay/fiddupay-admin-dashboard.git"

echo "Pushing 'admin-frontend' folder to $REMOTE_URL branch '$BRANCH'..."

# Use git subtree to push only the subfolder
if git subtree push --prefix admin-frontend "$REMOTE_URL" "$BRANCH"; then
  echo "✅ Admin Dashboard Code Push Successful!"
  
  if [ -n "$TAG" ]; then
    echo "🏷️  Pushing tag '$TAG' to $REMOTE_URL..."
    CURRENT_COMMIT=$(git rev-parse HEAD)
    if git push "$REMOTE_URL" "$CURRENT_COMMIT:refs/tags/$TAG"; then
      echo "✅ Admin Dashboard Tag Push Successful!"
    else
      echo "❌ Admin Dashboard Tag Push Failed."
    fi
  fi
else
  echo "❌ Admin Dashboard Code Push Failed. You might need to force push or handle conflicts."
  echo "Try running: git subtree split --prefix admin-frontend -b admin-split && git push $REMOTE_URL admin-split:$BRANCH --force && git branch -D admin-split"
fi

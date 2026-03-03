#!/bin/bash

# Script to push Main Frontend changes to the isolated Frontend repository
# Usage: ./scripts/push-frontend.sh [branch] [tag]
# bash scripts/push-frontend.sh main

BRANCH=${1:-main}
TAG=$2
# REPLACE WITH YOUR ACTUAL FRONTEND REPO URL
REMOTE_URL="https://github.com/fiddupay/fiddupay-frontend.git"

echo "Pushing 'frontend' folder to $REMOTE_URL branch '$BRANCH'..."

# Use git subtree to push only the subfolder
if git subtree push --prefix frontend "$REMOTE_URL" "$BRANCH"; then
  echo "✅ Frontend Code Push Successful!"
  
  if [ -n "$TAG" ]; then
    echo "🏷️  Pushing tag '$TAG' to $REMOTE_URL..."
    CURRENT_COMMIT=$(git rev-parse HEAD)
    if git push "$REMOTE_URL" "$CURRENT_COMMIT:refs/tags/$TAG"; then
      echo "✅ Frontend Tag Push Successful!"
    else
      echo "❌ Frontend Tag Push Failed."
    fi
  fi
else
  echo "❌ Frontend Code Push Failed. You might need to force push or handle conflicts."
  echo "Try running: git subtree split --prefix frontend -b frontend-split && git push $REMOTE_URL frontend-split:$BRANCH --force && git branch -D frontend-split"
fi

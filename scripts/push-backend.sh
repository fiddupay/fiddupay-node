#!/bin/bash

# Script to push Backend changes to the isolated Backend repository
# Usage: ./scripts/push-backend.sh [branch] [tag]
# bash scripts/push-backend.sh main

BRANCH=${1:-main}
TAG=$2
# REPLACE WITH YOUR ACTUAL BACKEND REPO URL
REMOTE_URL="https://github.com/fiddupay/fiddupay-backend.git"

echo "Pushing 'backend' folder to $REMOTE_URL branch '$BRANCH'..."

# Use git subtree to push only the subfolder
if git subtree push --prefix backend "$REMOTE_URL" "$BRANCH"; then
  echo "✅ Backend Code Push Successful!"
  
  if [ -n "$TAG" ]; then
    echo "🏷️  Pushing tag '$TAG' to $REMOTE_URL..."
    CURRENT_COMMIT=$(git rev-parse HEAD)
    if git push "$REMOTE_URL" "$CURRENT_COMMIT:refs/tags/$TAG"; then
      echo "✅ Backend Tag Push Successful!"
    else
      echo "❌ Backend Tag Push Failed."
    fi
  fi
else
  echo "❌ Backend Code Push Failed. You might need to force push or handle conflicts."
  echo "Try running: git subtree split --prefix backend -b backend-split && git push $REMOTE_URL backend-split:$BRANCH --force && git branch -D backend-split"
fi

#!/bin/bash

# Script to push P2P-Frontend changes to the isolated P2P repository
# Usage: ./scripts/push-p2p.sh [branch] [tag]
# bash scripts/push-p2p.sh main

BRANCH=${1:-main}
TAG=$2
# REPLACE WITH YOUR ACTUAL P2P REPO URL
REMOTE_URL="https://github.com/fiddupay/fiddupay-p2p.git"

echo "Pushing 'p2p-frontend' folder to $REMOTE_URL branch '$BRANCH'..."

# Use git subtree to push only the subfolder
if git subtree push --prefix p2p-frontend "$REMOTE_URL" "$BRANCH"; then
  echo "✅ P2P-Frontend Code Push Successful!"
  
  if [ -n "$TAG" ]; then
    echo "🏷️  Pushing tag '$TAG' to $REMOTE_URL..."
    CURRENT_COMMIT=$(git rev-parse HEAD)
    if git push "$REMOTE_URL" "$CURRENT_COMMIT:refs/tags/$TAG"; then
      echo "✅ P2P Tag Push Successful!"
    else
      echo "❌ P2P Tag Push Failed."
    fi
  fi
else
  echo "❌ P2P Code Push Failed. You might need to force push or handle conflicts."
  echo "Try running: git subtree split --prefix p2p-frontend -b p2p-split && git push $REMOTE_URL p2p-split:$BRANCH --force && git branch -D p2p-split"
fi

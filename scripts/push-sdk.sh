#!/bin/bash

# Script to push SDK changes to the isolated SDK repository
# Usage: git subtree push --prefix fiddupay-node-sdk https://github.com/fiddupay/fiddupay-node.git main

# Check if script is run as root/sudo
if [ "$EUID" -eq 0 ]; then
  echo "⚠️  Warning: Running this script with sudo/root is NOT recommended."
  echo "Git authentication usually fails when run as root if your credentials are saved for your normal user."
  echo "If you get 'permission denied', run: sudo chown -R \$USER:\$USER ."
  read -p "Do you still want to proceed? (y/N) " confirm
  if [[ ! $confirm =~ ^[Yy]$ ]]; then
    exit 1
  fi
fi

BRANCH=${1:-main}
TAG=$2
REMOTE_URL="https://github.com/fiddupay/fiddupay-node.git"

echo "Pushing 'fiddupay-node-sdk' folder to $REMOTE_URL branch '$BRANCH'..."

# Run npm audit fix inside the SDK folder before pushing
echo "🔍 Running npm audit fix in fiddupay-node-sdk..."
(cd fiddupay-node-sdk && npm audit fix)
if [ $? -ne 0 ]; then
  echo "⚠️  npm audit fix reported issues. Proceeding with push anyway (remaining issues may require manual review)."
fi

# Use git subtree to push only the subfolder
if git subtree push --prefix fiddupay-node-sdk "$REMOTE_URL" "$BRANCH"; then
  echo "✅ SDK Code Push Successful!"
  
  if [ -n "$TAG" ]; then
    echo "🏷️  Pushing tag '$TAG' to $REMOTE_URL..."
    # We need to push the tag to the remote. 
    # Since we can't easily 'subtree push' a tag, we'll push the current commit as a tag to the remote.
    CURRENT_COMMIT=$(git rev-parse HEAD)
    if git push "$REMOTE_URL" "$CURRENT_COMMIT:refs/tags/$TAG"; then
      echo "✅ SDK Tag Push Successful!"
    else
      echo "❌ SDK Tag Push Failed."
    fi
  fi
else
  echo "❌ SDK Code Push Failed. You might need to force push or handle conflicts."
  echo "Try running: git subtree split --prefix fiddupay-node-sdk -b sdk-split && git push $REMOTE_URL sdk-split:$BRANCH --force && git branch -D sdk-split"
fi

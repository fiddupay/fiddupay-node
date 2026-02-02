#!/bin/bash

# Script to push SDK changes to the isolated SDK repository
# Usage: ./scripts/push-sdk.sh [branch_name]

BRANCH=${1:-main}
REMOTE_URL="https://github.com/fiddupay/fiddupay-node.git"

echo "Pushing 'fiddupay-node-sdk' folder to $REMOTE_URL branch '$BRANCH'..."

# Use git subtree to push only the subfolder
if git subtree push --prefix fiddupay-node-sdk "$REMOTE_URL" "$BRANCH"; then
  echo "✅ SDK Push Successful!"
else
  echo "❌ SDK Push Failed. You might need to force push or handle conflicts."
  echo "Try running: git subtree split --prefix fiddupay-node-sdk -b sdk-split && git push $REMOTE_URL sdk-split:$BRANCH --force && git branch -D sdk-split"
fi

#!/bin/bash
# Fix dependency conflicts and build

echo "Fixing dependency conflicts..."

# Remove Cargo.lock to force fresh resolution
rm -f Cargo.lock

# Update all dependencies to latest compatible versions
cargo update

# Try building
echo "Building project..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed. Trying alternative fix..."
    
    # If still fails, try with specific dependency updates
    cargo update -p serde
    cargo update -p bitflags
    cargo update -p sqlx
    
    cargo build --release
fi

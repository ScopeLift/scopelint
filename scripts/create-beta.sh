#!/bin/bash

# Beta Release Script
# Creates a beta release with proper versioning

set -e

# Configuration
BIN_NAME="scopelint"
PROJECT_NAME="scopelint"

# Parse arguments
VERSION=${1:-"1.0.0"}
TAG_NAME="v${VERSION}-beta"

echo "🚀 Creating Beta Release"
echo "========================"
echo "Version: $VERSION"
echo "Tag: $TAG_NAME"
echo ""

# Step 1: Build with beta tag
echo "🔨 Building with beta version..."
GIT_TAG=beta cargo build --release

# Step 2: Commit any changes
echo "📝 Committing changes..."
git add .
git commit -m "Prepare for beta release $TAG_NAME" || true

# Step 3: Create git tag (force update if it already exists)
echo "🏷️  Creating git tag..."
if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    CURRENT_TAG_COMMIT=$(git rev-parse "$TAG_NAME")
    HEAD_COMMIT=$(git rev-parse HEAD)
    
    echo "⚠️  Tag $TAG_NAME already exists!"
    echo "   Current tag points to: $(git log -1 --oneline "$CURRENT_TAG_COMMIT")"
    echo "   HEAD is at:             $(git log -1 --oneline "$HEAD_COMMIT")"
    echo ""
    
    if [ "$CURRENT_TAG_COMMIT" != "$HEAD_COMMIT" ]; then
        read -p "Do you want to update the tag to point to HEAD? (y/N): " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "🔄 Updating tag to current commit..."
            git tag -f $TAG_NAME
            git push -f origin $TAG_NAME
        else
            echo "❌ Tag update cancelled. Keeping existing tag."
            exit 1
        fi
    else
        echo "✅ Tag already points to HEAD, no update needed."
        git push origin $TAG_NAME || echo "Tag already pushed, skipping..."
    fi
else
    git tag $TAG_NAME
    git push origin $TAG_NAME
fi

echo ""
echo "✅ Beta release created successfully!"
echo "📦 Tag: $TAG_NAME"
echo "🔗 Tag: https://github.com/ScopeLift/scopelint/releases/tag/$TAG_NAME"
echo ""
echo "💡 Team members can install with:"
echo "   ./scripts/install-beta.sh $TAG_NAME"

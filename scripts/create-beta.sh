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
if git diff --staged --quiet; then
    echo "   No changes to commit"
else
    git commit -m "Prepare for beta release $TAG_NAME"
    echo "   Changes committed"
fi

# Step 3: Create git tag at current HEAD
echo "🏷️  Creating git tag..."
CURRENT_COMMIT=$(git rev-parse --short HEAD)

# Check if tag exists locally or remotely
TAG_EXISTS_LOCALLY=false
TAG_EXISTS_REMOTELY=false

if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    TAG_EXISTS_LOCALLY=true
    OLD_COMMIT=$(git rev-parse --short "$TAG_NAME")
fi

if git ls-remote --tags origin "$TAG_NAME" 2>/dev/null | grep -q "$TAG_NAME"; then
    TAG_EXISTS_REMOTELY=true
    if [ "$TAG_EXISTS_LOCALLY" = false ]; then
        # Fetch to get remote tag info
        git fetch origin tag "$TAG_NAME" 2>/dev/null || true
        if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
            OLD_COMMIT=$(git rev-parse --short "$TAG_NAME")
        fi
    fi
fi

# Create or update tag locally
if [ "$TAG_EXISTS_LOCALLY" = true ]; then
    echo "   Tag $TAG_NAME already exists locally (points to $OLD_COMMIT)"
    echo "   Updating to point to $CURRENT_COMMIT"
    git tag -f $TAG_NAME HEAD
else
    git tag $TAG_NAME HEAD
    echo "   Tag $TAG_NAME created at $CURRENT_COMMIT"
fi

# Push tag with confirmation if it exists remotely
if [ "$TAG_EXISTS_REMOTELY" = true ]; then
    echo ""
    echo "⚠️  Tag $TAG_NAME already exists on remote (points to $OLD_COMMIT)"
    echo "   This will update it to point to $CURRENT_COMMIT"
    read -p "   Do you want to force push this tag? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git push origin $TAG_NAME --force
        echo "   Tag force pushed successfully"
    else
        echo "   Skipping tag push (tag not updated on remote)"
    fi
else
    git push origin $TAG_NAME
    echo "   Tag pushed successfully"
fi

echo ""
echo "✅ Beta release created successfully!"
echo "📦 Tag: $TAG_NAME"
echo "🔗 Tag: https://github.com/ScopeLift/scopelint/releases/tag/$TAG_NAME"
echo ""
echo "💡 Team members can install with:"
echo "   ./scripts/install-beta.sh $TAG_NAME"

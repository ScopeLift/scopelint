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

# Step 3: Create git tag
echo "🏷️  Creating git tag..."
git tag $TAG_NAME || echo "Tag $TAG_NAME already exists, skipping..."
git push origin $TAG_NAME || echo "Tag $TAG_NAME already pushed, skipping..."

echo ""
echo "✅ Beta release created successfully!"
echo "📦 Tag: $TAG_NAME"
echo "🔗 Tag: https://github.com/ScopeLift/scopelint/releases/tag/$TAG_NAME"
echo ""
echo "💡 Team members can install with:"
echo "   ./scripts/install-beta.sh $TAG_NAME"

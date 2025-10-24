#!/bin/bash

# Simple Beta Installer
# Downloads source from git tag and builds locally

set -e

# Configuration
REPO_NAME="ScopeLift/scopelint"
BIN_NAME="scopelint"
INSTALL_LOCATION="/usr/local/bin"

# Parse arguments
VERSION=${1:-"latest"}

echo "🚀 scopelint Beta Installer (Simple)"
echo "===================================="
echo "Repository: $REPO_NAME"
echo "Version: $VERSION"
echo "Install to: $INSTALL_LOCATION"
echo ""

# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd $TEMP_DIR

echo "📥 Downloading source from git tag..."

# Download source from git tag
curl -L -o source.tar.gz "https://github.com/$REPO_NAME/archive/refs/tags/$VERSION.tar.gz"
tar -xzf source.tar.gz
cd scopelint-*

echo "🔨 Building beta version..."

# Build with beta version
GIT_TAG=beta cargo build --release

echo "📦 Installing to $INSTALL_LOCATION..."

# Install the binary
sudo cp target/release/$BIN_NAME $INSTALL_LOCATION/
sudo chmod +x $INSTALL_LOCATION/$BIN_NAME

# Cleanup
cd /
rm -rf $TEMP_DIR

echo "✅ Installation complete!"

# Test the installation
echo "🧪 Testing installation..."
$BIN_NAME --version

echo ""
echo "🎉 Beta installation successful!"
echo "💡 You can now use '$BIN_NAME' in your projects"

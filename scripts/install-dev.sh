#!/bin/bash

# Local Development Installation Script
# Builds and installs scopelint from current directory

set -e

BIN_NAME="scopelint-dev"
INSTALL_LOCATION="$HOME/.local/bin"

echo "🔨 Building scopelint from source..."
echo "=================================="

# Build the binary (debug mode for local development)
cargo build

echo "📦 Installing to $INSTALL_LOCATION..."

# Create directory if it doesn't exist
mkdir -p "$INSTALL_LOCATION"

# Install the binary (rename it to scopelint-dev)
cp "target/debug/scopelint" "$INSTALL_LOCATION/$BIN_NAME"
chmod +x "$INSTALL_LOCATION/$BIN_NAME"

echo "✅ Installation complete!"

# Test the installation
echo "🧪 Testing installation..."
"$INSTALL_LOCATION/$BIN_NAME" --version
"$INSTALL_LOCATION/$BIN_NAME" --help

echo ""
echo "🎉 Local development build installed successfully!"
echo "💡 Local version installed as: $BIN_NAME"
echo "💡 Use '$BIN_NAME' for local development"
echo "💡 Use 'scopelint' for production version"
echo "🔄 To update, just run this script again after making changes"

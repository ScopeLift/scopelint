#!/bin/bash

# Beta Installation Script for scopelint
# Usage: ./scripts/install-beta.sh [version] [install-location]

set -e

# Configuration
REPO_NAME="ScopeLift/scopelint"
BIN_NAME="scopelint"
DEFAULT_VERSION="latest"
DEFAULT_INSTALL_LOCATION="/usr/local/bin"

# Parse arguments
VERSION=${1:-$DEFAULT_VERSION}
INSTALL_LOCATION=${2:-$DEFAULT_INSTALL_LOCATION}

echo "🚀 scopelint Beta Installer"
echo "=========================="
echo "Repository: $REPO_NAME"
echo "Version: $VERSION"
echo "Install to: $INSTALL_LOCATION"
echo ""

# Function to get the latest beta release
get_latest_beta() {
    echo "🔍 Finding latest beta release..."
    LATEST_BETA=$(gh release list --repo $REPO_NAME --limit 10 | grep -E "(beta|rc)" | head -1 | cut -f1)
    if [ -z "$LATEST_BETA" ]; then
        echo "❌ No beta releases found!"
        exit 1
    fi
    echo "📦 Found latest beta: $LATEST_BETA"
    echo $LATEST_BETA
}

# Function to download and install
install_binary() {
    local version=$1
    local install_path=$2
    
    echo "📥 Downloading $BIN_NAME $version..."
    
    # Create temporary directory
    TEMP_DIR=$(mktemp -d)
    cd $TEMP_DIR
    
    # Download the release
    if [ "$version" = "latest" ]; then
        # Get the latest beta release
        version=$(get_latest_beta)
    fi
    
    # Download the binary
    wget -q "https://github.com/$REPO_NAME/releases/download/$version/scopelint-linux-x86_64.tar.gz"
    
    # Extract
    tar -xzf scopelint-linux-x86_64.tar.gz
    
    # Install
    echo "📦 Installing to $install_path..."
    sudo mv $BIN_NAME $install_path/
    sudo chmod +x $install_path/$BIN_NAME
    
    # Cleanup
    cd /
    rm -rf $TEMP_DIR
    
    echo "✅ Installation complete!"
}

# Function to test the installation
test_installation() {
    echo "🧪 Testing installation..."
    
    if command -v $BIN_NAME &> /dev/null; then
        echo "✅ $BIN_NAME is available in PATH"
        echo "📋 Version info:"
        $BIN_NAME --version
        echo ""
        echo "📋 Help info:"
        $BIN_NAME --help
    else
        echo "❌ $BIN_NAME not found in PATH"
        echo "💡 Make sure $INSTALL_LOCATION is in your PATH"
        exit 1
    fi
}

# Main execution
main() {
    # Check if gh CLI is installed
    if ! command -v gh &> /dev/null; then
        echo "❌ GitHub CLI (gh) is required but not installed."
        echo "💡 Install it with: brew install gh (macOS) or apt install gh (Ubuntu)"
        exit 1
    fi
    
    # Check if we're authenticated with GitHub
    if ! gh auth status &> /dev/null; then
        echo "❌ Not authenticated with GitHub CLI"
        echo "💡 Run: gh auth login"
        exit 1
    fi
    
    install_binary $VERSION $INSTALL_LOCATION
    test_installation
    
    echo ""
    echo "🎉 Beta installation successful!"
    echo "💡 You can now use '$BIN_NAME' in your projects"
    echo "🔄 To update, run this script again with a new version"
}

# Run main function
main "$@"

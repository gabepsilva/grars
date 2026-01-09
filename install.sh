#!/bin/bash
# Detect OS and call appropriate installer
# This script downloads required install scripts from GitHub if they don't exist locally

set -euo pipefail

OS=$(uname -s)
# Handle case where script is piped from curl (BASH_SOURCE[0] may be unbound)
if [ -n "${BASH_SOURCE[0]:-}" ]; then
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
else
    # Script is being piped, use current directory or home directory
    SCRIPT_DIR="${PWD:-$HOME}"
fi
GITHUB_REPO="${GITHUB_REPO:-gabepsilva/grars}"

# Use cache directory for downloaded scripts (or local install directory if in repo)
if [ -d "$SCRIPT_DIR/install" ] && [ -f "$SCRIPT_DIR/install/common-bash.sh" ]; then
    # We're in the repository, use local scripts
    INSTALL_DIR="$SCRIPT_DIR/install"
else
    # Download to cache directory
    INSTALL_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/grars-install"
    mkdir -p "$INSTALL_DIR"
fi

# Function to download file from GitHub
download_file() {
    local url="$1"
    local output="$2"
    
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$output" "$url"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$output" "$url"
    else
        return 1
    fi
}

# Ensure install directory exists (already created above if using cache)

# Download required scripts from GitHub if they don't exist locally
GITHUB_BASE="https://raw.githubusercontent.com/$GITHUB_REPO/master/install"

# Check and download common-bash.sh
if [ ! -f "$INSTALL_DIR/common-bash.sh" ]; then
    echo "Downloading common-bash.sh from GitHub..."
    if ! download_file "$GITHUB_BASE/common-bash.sh" "$INSTALL_DIR/common-bash.sh"; then
        echo "Error: Failed to download common-bash.sh from GitHub"
        exit 1
    fi
    chmod +x "$INSTALL_DIR/common-bash.sh"
fi

# Check and download platform-specific install script
case "$OS" in
    Linux)
        if [ ! -f "$INSTALL_DIR/install-linux.sh" ]; then
            echo "Downloading install-linux.sh from GitHub..."
            if ! download_file "$GITHUB_BASE/install-linux.sh" "$INSTALL_DIR/install-linux.sh"; then
                echo "Error: Failed to download install-linux.sh from GitHub"
                exit 1
            fi
            chmod +x "$INSTALL_DIR/install-linux.sh"
        fi
        exec "$INSTALL_DIR/install-linux.sh" "$@"
        ;;
    Darwin)
        if [ ! -f "$INSTALL_DIR/install-macos.sh" ]; then
            echo "Downloading install-macos.sh from GitHub..."
            if ! download_file "$GITHUB_BASE/install-macos.sh" "$INSTALL_DIR/install-macos.sh"; then
                echo "Error: Failed to download install-macos.sh from GitHub"
                exit 1
            fi
            chmod +x "$INSTALL_DIR/install-macos.sh"
        fi
        exec "$INSTALL_DIR/install-macos.sh" "$@"
        ;;
    *)
        echo "Unsupported OS: $OS"
        echo "Please use the appropriate installer script directly:"
        echo "  - Linux: install/install-linux.sh"
        echo "  - macOS: install/install-macos.sh"
        exit 1
        ;;
esac


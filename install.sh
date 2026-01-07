#!/bin/bash
# Installation script for grars
# Supports multiple Linux distributions: Arch, Debian/Ubuntu, Fedora, openSUSE, Alpine, Void, etc.
#
# Install directly via URL:
#   curl -fsSL https://raw.githubusercontent.com/gabepsilva/grars/main/install.sh | bash
#
# This script will:
#   1. Detect your Linux distribution
#   2. Install system dependencies (espeak-ng, Python3, venv, wl-clipboard/xclip)
#   3. Install grars binary (tries local build first, then downloads from GitHub)
#   4. Install grars to ~/.local/bin
#   5. Set up piper-tts dependencies (venv, piper-tts package, voice models)
#
# Usage:
#   # Direct installation via URL (recommended)
#   curl -fsSL https://raw.githubusercontent.com/gabepsilva/grars/main/install.sh | bash
#   # or
#   wget -qO- https://raw.githubusercontent.com/gabepsilva/grars/main/install.sh | bash
#
#   # Or download and run locally
#   ./install.sh
#
# Requirements:
#   - sudo access (for installing system packages)
#   - Internet connection (for downloading binary, packages and models)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Installation directories (XDG Base Directory standard)
INSTALL_DIR="$HOME/.local/share/grars"
BIN_DIR="$HOME/.local/bin"
VENV_DIR="$INSTALL_DIR/venv"
MODELS_DIR="$INSTALL_DIR/models"
GRARS_BIN="$BIN_DIR/grars"

# GitHub repository
GITHUB_REPO="${GITHUB_REPO:-gabepsilva/grars}"
GITHUB_API="https://api.github.com/repos/$GITHUB_REPO"
GRARS_VERSION="${GRARS_VERSION:-1.0.0}"

log_info "Installing to: $INSTALL_DIR"
log_info "Binary will be installed to: $BIN_DIR"

# Model to download (default)
# Note: Models are downloaded from HuggingFace main branch (always latest)
MODEL_NAME="en_US-lessac-medium"

# Detect Linux distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        # Source os-release, temporarily disable unbound variable check
        set +u
        . /etc/os-release
        DISTRO_ID="${ID:-unknown}"
        DISTRO_ID_LIKE="${ID_LIKE:-}"
        set -u
    elif [ -f /etc/arch-release ]; then
        DISTRO_ID="arch"
    elif [ -f /etc/debian_version ]; then
        DISTRO_ID="debian"
    elif [ -f /etc/redhat-release ]; then
        DISTRO_ID="rhel"
    else
        DISTRO_ID="unknown"
    fi
    log_info "Detected distribution: $DISTRO_ID"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download file using curl or wget (whichever is available)
download_file() {
    local url="$1"
    local output="$2"
    
    if command_exists curl; then
        curl -fsSL -o "$output" "$url"
    elif command_exists wget; then
        wget -q -O "$output" "$url"
    else
        return 1
    fi
}

# Check all required dependencies and install if missing
check_and_install_dependencies() {
    local missing_deps=()
    local packages_to_install=()
    
    log_info "Checking required dependencies..."
    
    # Check espeak-ng
    if ! command_exists espeak-ng; then
        missing_deps+=("espeak-ng")
        log_warn "espeak-ng not found (required)"
    else
        log_success "espeak-ng found"
    fi
    
    # Check clipboard utilities (need at least one for Wayland or X11)
    local clipboard_available=false
    if command_exists wl-paste; then
        log_success "wl-paste found (Wayland clipboard support)"
        clipboard_available=true
    else
        log_warn "wl-paste not found (Wayland clipboard support missing)"
    fi
    
    if command_exists xclip; then
        log_success "xclip found (X11 clipboard support)"
        clipboard_available=true
    else
        log_warn "xclip not found (X11 clipboard support missing)"
    fi
    
    if [ "$clipboard_available" = false ]; then
        missing_deps+=("clipboard-utils")
        log_warn "No clipboard utility found (need wl-clipboard for Wayland or xclip for X11)"
    fi
    
    # Check Python3
    local python_missing=false
    local venv_missing=false
    if ! command_exists python3; then
        missing_deps+=("python3")
        python_missing=true
        log_warn "python3 not found (required)"
    else
        PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
        log_info "Python3 found: $PYTHON_VERSION"
        
        # Check venv module - try to actually use it, not just check help
        if ! python3 -m venv --help >/dev/null 2>&1; then
            missing_deps+=("python3-venv")
            venv_missing=true
            log_warn "python3-venv module not found (required)"
        else
            # Test if venv can actually create a venv (requires ensurepip)
            local test_venv_dir
            test_venv_dir=$(mktemp -d)
            if python3 -m venv "$test_venv_dir" >/dev/null 2>&1; then
                rm -rf "$test_venv_dir"
                log_success "Python3 venv module is available"
            else
                missing_deps+=("python3-venv")
                venv_missing=true
                log_warn "python3-venv module cannot create virtual environments (required)"
                rm -rf "$test_venv_dir" 2>/dev/null || true
            fi
        fi
    fi
    
    # If all dependencies are present, return
    if [ ${#missing_deps[@]} -eq 0 ]; then
        log_success "All required dependencies are installed"
        return 0
    fi
    
    # Show missing dependencies and ask user
    echo ""
    log_warn "Missing required dependencies:"
    for dep in "${missing_deps[@]}"; do
        echo "  - $dep"
    done
    echo ""
    read -p "Install missing dependencies? [Y/n] " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        log_error "Cannot continue without required dependencies"
        exit 1
    fi
    
    # Determine packages to install based on distribution
    case "$DISTRO_ID" in
        arch|manjaro|endeavouros)
            if command_exists pacman; then
                [ "$python_missing" = true ] && packages_to_install+=("python" "python-pip")
                [[ " ${missing_deps[@]} " =~ " espeak-ng " ]] && packages_to_install+=("espeak-ng")
                [[ " ${missing_deps[@]} " =~ " clipboard-utils " ]] && packages_to_install+=("wl-clipboard" "xclip")
                if [ ${#packages_to_install[@]} -gt 0 ]; then
                    log_info "Installing packages via pacman: ${packages_to_install[*]}"
                    sudo pacman -S --needed --noconfirm "${packages_to_install[@]}"
                fi
            else
                log_error "pacman not found. Please install dependencies manually."
                exit 1
            fi
            ;;
        debian|ubuntu|linuxmint|pop)
            if command_exists apt-get; then
                [ "$python_missing" = true ] && packages_to_install+=("python3" "python3-venv" "python3-pip")
                [ "$venv_missing" = true ] && packages_to_install+=("python3-venv")
                [[ " ${missing_deps[@]} " =~ " espeak-ng " ]] && packages_to_install+=("espeak-ng")
                [[ " ${missing_deps[@]} " =~ " clipboard-utils " ]] && packages_to_install+=("wl-clipboard" "xclip")
                # Remove duplicates
                IFS=" " read -r -a packages_to_install <<< "$(printf '%s\n' "${packages_to_install[@]}" | sort -u | tr '\n' ' ')"
                if [ ${#packages_to_install[@]} -gt 0 ]; then
                    log_info "Installing packages via apt: ${packages_to_install[*]}"
                    sudo apt-get update
                    sudo apt-get install -y "${packages_to_install[@]}"
                fi
            else
                log_error "apt-get not found. Please install dependencies manually."
                exit 1
            fi
            ;;
        fedora|rhel|centos|rocky|almalinux)
            if command_exists dnf; then
                [ "$python_missing" = true ] && packages_to_install+=("python3" "python3-pip")
                [[ " ${missing_deps[@]} " =~ " espeak-ng " ]] && packages_to_install+=("espeak-ng")
                [[ " ${missing_deps[@]} " =~ " clipboard-utils " ]] && packages_to_install+=("wl-clipboard" "xclip")
                if [ ${#packages_to_install[@]} -gt 0 ]; then
                    log_info "Installing packages via dnf: ${packages_to_install[*]}"
                    sudo dnf install -y "${packages_to_install[@]}"
                fi
            elif command_exists yum; then
                [ "$python_missing" = true ] && packages_to_install+=("python3" "python3-pip")
                [[ " ${missing_deps[@]} " =~ " espeak-ng " ]] && packages_to_install+=("espeak-ng")
                [[ " ${missing_deps[@]} " =~ " clipboard-utils " ]] && packages_to_install+=("wl-clipboard" "xclip")
                if [ ${#packages_to_install[@]} -gt 0 ]; then
                    log_info "Installing packages via yum: ${packages_to_install[*]}"
                    sudo yum install -y "${packages_to_install[@]}"
                fi
            else
                log_error "dnf/yum not found. Please install dependencies manually."
                exit 1
            fi
            ;;
        opensuse*|sles)
            if command_exists zypper; then
                [ "$python_missing" = true ] && packages_to_install+=("python3" "python3-pip")
                [[ " ${missing_deps[@]} " =~ " espeak-ng " ]] && packages_to_install+=("espeak-ng")
                [[ " ${missing_deps[@]} " =~ " clipboard-utils " ]] && packages_to_install+=("wl-clipboard" "xclip")
                if [ ${#packages_to_install[@]} -gt 0 ]; then
                    log_info "Installing packages via zypper: ${packages_to_install[*]}"
                    sudo zypper install -y "${packages_to_install[@]}"
                fi
            else
                log_error "zypper not found. Please install dependencies manually."
                exit 1
            fi
            ;;
        alpine)
            if command_exists apk; then
                [ "$python_missing" = true ] && packages_to_install+=("python3" "py3-pip")
                [[ " ${missing_deps[@]} " =~ " espeak-ng " ]] && packages_to_install+=("espeak-ng")
                [[ " ${missing_deps[@]} " =~ " clipboard-utils " ]] && packages_to_install+=("wl-clipboard" "xclip")
                if [ ${#packages_to_install[@]} -gt 0 ]; then
                    log_info "Installing packages via apk: ${packages_to_install[*]}"
                    sudo apk add --no-cache "${packages_to_install[@]}"
                fi
            else
                log_error "apk not found. Please install dependencies manually."
                exit 1
            fi
            ;;
        void)
            if command_exists xbps-install; then
                [ "$python_missing" = true ] && packages_to_install+=("python3")
                [[ " ${missing_deps[@]} " =~ " espeak-ng " ]] && packages_to_install+=("espeak-ng")
                [[ " ${missing_deps[@]} " =~ " clipboard-utils " ]] && packages_to_install+=("wl-clipboard" "xclip")
                if [ ${#packages_to_install[@]} -gt 0 ]; then
                    log_info "Installing packages via xbps-install: ${packages_to_install[*]}"
                    sudo xbps-install -S -y "${packages_to_install[@]}"
                fi
            else
                log_error "xbps-install not found. Please install dependencies manually."
                exit 1
            fi
            ;;
        *)
            log_error "Unsupported distribution: $DISTRO_ID"
            log_warn "Please install required dependencies manually, then run this script again."
            exit 1
            ;;
    esac
    
    # Verify installations
    if ! command_exists python3; then
        log_error "Python3 installation failed or not found in PATH"
        exit 1
    fi
    
    # Verify venv can actually create a venv
    local test_venv_dir
    test_venv_dir=$(mktemp -d)
    if ! python3 -m venv "$test_venv_dir" >/dev/null 2>&1; then
        rm -rf "$test_venv_dir" 2>/dev/null || true
        log_error "Python3 venv module cannot create virtual environments"
        log_error "On Debian/Ubuntu, you may need: sudo apt install python3-venv"
        exit 1
    fi
    rm -rf "$test_venv_dir"
    log_success "Python3 venv module verified"
    
    if ! command_exists espeak-ng; then
        log_warn "espeak-ng installation may have failed. Piper may not work correctly."
    fi
    
    # Verify clipboard utilities
    local clipboard_ok=false
    if command_exists wl-paste; then
        log_success "wl-paste verified (Wayland clipboard support)"
        clipboard_ok=true
    fi
    if command_exists xclip; then
        log_success "xclip verified (X11 clipboard support)"
        clipboard_ok=true
    fi
    if [ "$clipboard_ok" = false ]; then
        log_warn "Clipboard utilities installation may have failed. App may not be able to read selected text."
    fi
    
    log_success "Dependencies installed successfully"
}

# Create virtual environment
create_venv() {
    log_info "Creating virtual environment at $VENV_DIR..."
    
    # Remove existing venv if it exists
    if [ -d "$VENV_DIR" ]; then
        log_warn "Existing venv found at $VENV_DIR. Removing..."
        rm -rf "$VENV_DIR"
    fi
    
    # Create parent directory
    mkdir -p "$INSTALL_DIR"
    
    # Create venv
    python3 -m venv "$VENV_DIR"
    
    if [ ! -f "$VENV_DIR/bin/activate" ]; then
        log_error "Failed to create virtual environment"
        exit 1
    fi
    
    log_success "Virtual environment created"
}

# Install piper-tts in venv
install_piper() {
    log_info "Installing piper-tts in virtual environment..."
    
    # Activate venv and install
    source "$VENV_DIR/bin/activate"
    
    # Upgrade pip first
    log_info "Upgrading pip..."
    pip install --quiet --upgrade pip
    
    # Clear pip cache to avoid dependency conflicts (especially on Fedora)
    log_info "Clearing pip cache..."
    pip cache purge 2>/dev/null || true
    
    # Install onnxruntime first (required dependency for piper-tts)
    # This helps with dependency resolution, especially on Python 3.14+
    log_info "Installing onnxruntime (required dependency)..."
    if ! pip install --quiet "onnxruntime<2,>=1"; then
        log_warn "Standard onnxruntime installation failed, trying nightly build..."
        log_info "Nightly builds support newer Python versions (e.g., 3.14+)"
        if ! pip install --quiet --pre onnxruntime \
            --extra-index-url=https://aiinfra.pkgs.visualstudio.com/PublicPackages/_packaging/ORT-Nightly/pypi/simple/; then
            log_error "Failed to install onnxruntime (required by piper-tts)"
            log_error "This may be due to Python version incompatibility"
            deactivate
            exit 1
        else
            log_success "onnxruntime nightly build installed successfully"
        fi
    else
        log_success "onnxruntime installed successfully"
    fi
    
    # Install piper-tts
    # Since we already have onnxruntime installed, try installing piper-tts
    # First try normal installation, then try without dependency checks
    log_info "Installing piper-tts package..."
    if ! pip install --quiet --upgrade --force-reinstall piper-tts; then
        log_warn "Standard installation failed, trying without dependency checks..."
        log_info "Installing piper-tts without dependency resolution (deps already installed)..."
        # Install piper-tts without checking dependencies since we have onnxruntime
        if ! pip install --quiet --upgrade --force-reinstall --no-deps piper-tts; then
            log_error "Failed to install piper-tts"
            deactivate
            exit 1
        fi
        # Install other piper-tts dependencies that might be missing
        log_info "Installing piper-tts dependencies..."
        pip install --quiet piper-phonemize || true
    fi
    
    # Verify installation
    if [ ! -f "$VENV_DIR/bin/piper" ]; then
        log_error "piper binary not found after installation"
        deactivate
        exit 1
    fi
    
    # Test piper (--help is more reliable than --version)
    if "$VENV_DIR/bin/piper" --help >/dev/null 2>&1; then
        # Try to get version, but don't fail if it doesn't work
        PIPER_VERSION=$("$VENV_DIR/bin/piper" --version 2>&1 | head -1 2>/dev/null || echo "installed")
        log_success "piper-tts installed successfully"
        if [ "$PIPER_VERSION" != "installed" ]; then
            log_info "Piper version: $PIPER_VERSION"
        fi
    else
        log_error "piper binary found but doesn't respond to --help"
        deactivate
        exit 1
    fi
    
    deactivate
}

# Download Piper model
download_model() {
    log_info "Checking for model: $MODEL_NAME..."
    
    MODEL_ONNX="$MODELS_DIR/$MODEL_NAME.onnx"
    MODEL_JSON="$MODELS_DIR/$MODEL_NAME.onnx.json"
    
    # Check if model already exists
    if [ -f "$MODEL_ONNX" ] && [ -f "$MODEL_JSON" ]; then
        log_success "Model already exists at $MODELS_DIR"
        return 0
    fi
    
    log_info "Model not found. Downloading from HuggingFace..."
    
    # Create models directory
    mkdir -p "$MODELS_DIR"
    
    # Use the correct HuggingFace URL structure (from dad project)
    # Format: https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx
    MODEL_BASE_URL="https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium"
    
    cd "$MODELS_DIR" || {
        log_error "Failed to change to models directory: $MODELS_DIR"
        return 1
    }
    
    # Download model files
    log_info "Downloading $MODEL_NAME.onnx..."
    if download_file "$MODEL_BASE_URL/$MODEL_NAME.onnx" "$MODEL_NAME.onnx"; then
        log_info "Downloading $MODEL_NAME.onnx.json..."
        if download_file "$MODEL_BASE_URL/$MODEL_NAME.onnx.json" "$MODEL_NAME.onnx.json"; then
            if [ -f "$MODEL_NAME.onnx" ] && [ -f "$MODEL_NAME.onnx.json" ]; then
                log_success "Model downloaded successfully to $MODELS_DIR"
                cd - >/dev/null || true
                return 0
            fi
        fi
    fi
    # Cleanup on failure
    rm -f "$MODEL_NAME.onnx" "$MODEL_NAME.onnx.json"
    if ! command_exists curl && ! command_exists wget; then
        log_error "Neither wget nor curl found. Please install one to download models."
    else
        log_error "Failed to download model files"
    fi
    cd - >/dev/null || true
    return 1
    
    # If download failed, provide manual instructions
    log_warn "Automatic model download failed"
    log_info "Please download the model manually from:"
    log_info "  $MODEL_BASE_URL/$MODEL_NAME.onnx"
    log_info "  $MODEL_BASE_URL/$MODEL_NAME.onnx.json"
    log_info ""
    log_info "Or visit: https://huggingface.co/rhasspy/piper-voices"
    log_info ""
    log_info "Place the files in: $MODELS_DIR"
    log_info "  - $MODEL_NAME.onnx"
    log_info "  - $MODEL_NAME.onnx.json"
    return 1
}

# Detect system OS
detect_os() {
    local os
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "$os" in
        linux)
            OS="linux"
            ;;
        darwin)
            OS="macos"
            ;;
        *)
            OS="linux"  # Default fallback
            log_warn "Unknown OS $os, defaulting to linux"
            ;;
    esac
    log_info "Detected OS: $OS"
}

# Detect system architecture
detect_arch() {
    local arch
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        armv7l|armv7)
            ARCH="armv7"
            ;;
        *)
            ARCH="x86_64"  # Default fallback
            log_warn "Unknown architecture $arch, defaulting to x86_64"
            ;;
    esac
    log_info "Detected architecture: $ARCH"
}

# Get latest release version from GitHub
get_latest_release() {
    log_info "Fetching latest release from GitHub..."
    
    local temp_file
    temp_file=$(mktemp)
    if download_file "$GITHUB_API/releases/latest" "$temp_file" 2>/dev/null; then
        LATEST_RELEASE=$(grep '"tag_name":' "$temp_file" 2>/dev/null | sed -E 's/.*"([^"]+)".*/\1/' || echo "")
        rm -f "$temp_file"
    else
        LATEST_RELEASE=""
        rm -f "$temp_file"
    fi
    
    if [ -z "$LATEST_RELEASE" ]; then
        log_warn "Failed to fetch latest release. Using 'latest' tag."
        LATEST_RELEASE="latest"
    else
        log_info "Latest release: $LATEST_RELEASE"
    fi
}

# Download and install grars binary from GitHub
download_and_install_binary() {
    log_info "Downloading grars binary from GitHub..."
    
    # Ensure bin directory exists
    mkdir -p "$BIN_DIR"
    
    # Detect OS and architecture
    detect_os
    detect_arch
    
    # Construct binary name: grars-1.0.0-linux-x86_64
    BINARY_NAME="grars-${GRARS_VERSION}-${OS}-${ARCH}"
    
    # Use specific release tag for v1.0.0, or allow override via RELEASE_TAG env var
    RELEASE_TAG="${RELEASE_TAG:-v1.0.0}"
    DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/download/$RELEASE_TAG/$BINARY_NAME"
    
    # Download binary
    if download_file "$DOWNLOAD_URL" "$GRARS_BIN"; then
        chmod +x "$GRARS_BIN"
        log_success "Binary downloaded and installed to $GRARS_BIN"
        return 0
    else
        if ! command_exists curl && ! command_exists wget; then
            log_error "Neither curl nor wget found. Please install one."
        else
            log_error "Failed to download binary from $DOWNLOAD_URL"
        fi
        return 1
    fi
}

# Install grars binary (try local first, then download from GitHub)
install_binary() {
    log_info "Installing grars binary..."
    
    # Ensure bin directory exists
    mkdir -p "$BIN_DIR"
    
    # Try to copy from local target/release directory
    local local_binary
    local_binary=""
    
    # Check if we're in the project directory
    if [ -f "Cargo.toml" ] && [ -d "target/release" ]; then
        if [ -f "target/release/grars" ]; then
            local_binary="target/release/grars"
            log_info "Found local build in target/release/grars"
        fi
    fi
    
    # Also check current directory
    if [ -z "$local_binary" ] && [ -f "grars" ] && [ -x "grars" ]; then
        local_binary="grars"
        log_info "Found grars binary in current directory"
    fi
    
    # If local binary found, copy it
    if [ -n "$local_binary" ]; then
        log_info "Copying binary from $local_binary to $GRARS_BIN"
        cp "$local_binary" "$GRARS_BIN"
        chmod +x "$GRARS_BIN"
        log_success "Binary copied and installed to $GRARS_BIN"
        return 0
    fi
    
    # No local binary found, try downloading from GitHub
    log_info "No local binary found. Attempting to download from GitHub..."
    if download_and_install_binary; then
        return 0
    fi
    
    # Both methods failed
    log_error "Failed to install binary"
    log_info "Please build the binary first: cargo build --release"
    log_info "Or place a grars binary in the current directory"
    return 1
}

# Install desktop file and icon
install_desktop() {
    DESKTOP_DIR="$HOME/.local/share/applications"
    ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
    DESKTOP_FILE="$DESKTOP_DIR/grars.desktop"
    ICON_FILE="$ICON_DIR/grars.svg"
    
    # Get script directory (or current directory if script is not available)
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || pwd)"
    
    # Create directories
    mkdir -p "$DESKTOP_DIR"
    mkdir -p "$ICON_DIR"
    
    # Install desktop file if available
    if [ -f "$SCRIPT_DIR/grars.desktop" ]; then
        sed "s#\\\$HOME#$HOME#g" "$SCRIPT_DIR/grars.desktop" > "$DESKTOP_FILE"
        chmod 644 "$DESKTOP_FILE"
        log_success "Desktop file installed to $DESKTOP_FILE"
        
        # Update desktop database if available
        if command -v update-desktop-database >/dev/null 2>&1; then
            update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
            log_info "Desktop database updated"
        fi
    else
        log_warn "Desktop file not found at $SCRIPT_DIR/grars.desktop (skipping)"
    fi
    
    # Install icon if available
    if [ -f "$SCRIPT_DIR/assets/logo.svg" ]; then
        cp "$SCRIPT_DIR/assets/logo.svg" "$ICON_FILE"
        log_success "Icon installed to $ICON_FILE"
        
        # Update icon cache if available
        if command -v gtk-update-icon-cache >/dev/null 2>&1; then
            gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
            log_info "Icon cache updated"
        fi
    else
        log_warn "Icon not found at $SCRIPT_DIR/assets/logo.svg (skipping)"
    fi
}

# Main installation function
main() {
    echo "=========================================="
    echo "  grars Installation Script"
    echo "=========================================="
    echo ""
    
    detect_distro
    check_and_install_dependencies
    install_binary
    create_venv
    install_piper
    
    # Download model if not present (download_model checks if it exists first)
    echo ""
    download_model
    
    # Install desktop file and icon
    echo ""
    install_desktop
    
    echo ""
    log_success "Installation complete!"
    echo ""
    echo "grars binary: $GRARS_BIN"
    echo "Piper venv: $VENV_DIR/bin/piper"
    echo "Models directory: $MODELS_DIR"
    echo ""
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        log_warn "$HOME/.local/bin is not in your PATH"
        echo "Add this to your shell configuration (~/.bashrc, ~/.zshrc, etc.):"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
    fi
    echo "Run grars with: grars"
    echo ""
}

# Run main function
main "$@"


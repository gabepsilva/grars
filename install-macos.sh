#!/bin/bash

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

# Installation directories (macOS standard locations)
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

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download file using curl (macOS has curl by default)
download_file() {
    local url="$1"
    local output="$2"
    
    if command_exists curl; then
        curl -fsSL -o "$output" "$url"
    else
        log_error "curl not found (should be available on macOS)"
        return 1
    fi
}

# Check if Homebrew is installed
check_homebrew() {
    # First check if brew is in PATH
    if command_exists brew; then
        log_success "Homebrew found"
        return 0
    fi
    
    # Try to find brew in common locations and add to PATH
    if [ -f "/opt/homebrew/bin/brew" ]; then
        export PATH="/opt/homebrew/bin:$PATH"
        if command_exists brew; then
            log_success "Homebrew found (added to PATH)"
            # Also add to shell config for future sessions
            if [ -f "$HOME/.bash_profile" ]; then
                if ! grep -q "/opt/homebrew/bin" "$HOME/.bash_profile"; then
                    echo 'export PATH="/opt/homebrew/bin:$PATH"' >> "$HOME/.bash_profile"
                fi
            elif [ -f "$HOME/.bashrc" ]; then
                if ! grep -q "/opt/homebrew/bin" "$HOME/.bashrc"; then
                    echo 'export PATH="/opt/homebrew/bin:$PATH"' >> "$HOME/.bashrc"
                fi
            fi
            return 0
        fi
    elif [ -f "/usr/local/bin/brew" ]; then
        export PATH="/usr/local/bin:$PATH"
        if command_exists brew; then
            log_success "Homebrew found (added to PATH)"
            # Also add to shell config for future sessions
            if [ -f "$HOME/.bash_profile" ]; then
                if ! grep -q "/usr/local/bin" "$HOME/.bash_profile"; then
                    echo 'export PATH="/usr/local/bin:$PATH"' >> "$HOME/.bash_profile"
                fi
            elif [ -f "$HOME/.bashrc" ]; then
                if ! grep -q "/usr/local/bin" "$HOME/.bashrc"; then
                    echo 'export PATH="/usr/local/bin:$PATH"' >> "$HOME/.bashrc"
                fi
            fi
            return 0
        fi
    fi
    
    # Homebrew not found, offer to install
    log_warn "Homebrew not found"
    log_info "Homebrew is required to install dependencies on macOS"
    log_info "Install it from: https://brew.sh"
    echo ""
        read -p "Install Homebrew now? [Y/n] " -r
    echo ""
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        log_error "Cannot continue without Homebrew"
        exit 1
    fi
    
    log_info "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add Homebrew to PATH (Apple Silicon uses /opt/homebrew, Intel uses /usr/local)
    if [ -d "/opt/homebrew/bin" ]; then
        export PATH="/opt/homebrew/bin:$PATH"
        # Also add to shell config for future sessions
        if [ -f "$HOME/.bash_profile" ]; then
            if ! grep -q "/opt/homebrew/bin" "$HOME/.bash_profile"; then
                echo 'export PATH="/opt/homebrew/bin:$PATH"' >> "$HOME/.bash_profile"
            fi
        elif [ -f "$HOME/.bashrc" ]; then
            if ! grep -q "/opt/homebrew/bin" "$HOME/.bashrc"; then
                echo 'export PATH="/opt/homebrew/bin:$PATH"' >> "$HOME/.bashrc"
            fi
        fi
    elif [ -d "/usr/local/bin" ]; then
        export PATH="/usr/local/bin:$PATH"
        # Also add to shell config for future sessions
        if [ -f "$HOME/.bash_profile" ]; then
            if ! grep -q "/usr/local/bin" "$HOME/.bash_profile"; then
                echo 'export PATH="/usr/local/bin:$PATH"' >> "$HOME/.bash_profile"
            fi
        elif [ -f "$HOME/.bashrc" ]; then
            if ! grep -q "/usr/local/bin" "$HOME/.bashrc"; then
                echo 'export PATH="/usr/local/bin:$PATH"' >> "$HOME/.bashrc"
            fi
        fi
    fi
    
    # Verify Homebrew installation
    if ! command_exists brew; then
        log_error "Homebrew installation failed or not in PATH"
        log_info "Please restart your terminal and run this script again"
        exit 1
    fi
    
    log_success "Homebrew installed successfully"
}

# Check all required dependencies and install if missing
check_and_install_dependencies() {
    local missing_deps=()
    
    log_info "Checking required dependencies..."
    
    # Check espeak-ng
    if ! command_exists espeak-ng; then
        missing_deps+=("espeak-ng")
        log_warn "espeak-ng not found (required)"
    else
        log_success "espeak-ng found"
    fi
    
    # Check clipboard utilities (macOS has pbpaste built-in)
    if command_exists pbpaste; then
        log_success "pbpaste found (macOS clipboard support)"
    else
        log_warn "pbpaste not found (unusual - should be built-in on macOS)"
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
        read -p "Install missing dependencies via Homebrew? [Y/n] " -r
    echo ""
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        log_error "Cannot continue without required dependencies"
        exit 1
    fi
    
    # Ensure Homebrew is available
    check_homebrew
    
    # Install packages via Homebrew
    local packages_to_install=()
    
    if [ "$python_missing" = true ]; then
        packages_to_install+=("python@3.12")
    fi
    
    if [[ " ${missing_deps[@]} " =~ " espeak-ng " ]]; then
        packages_to_install+=("espeak-ng")
    fi
    
    if [ ${#packages_to_install[@]} -gt 0 ]; then
        log_info "Installing packages via Homebrew: ${packages_to_install[*]}"
        brew install "${packages_to_install[@]}"
    fi
    
    # Verify installations
    if ! command_exists python3; then
        log_error "Python3 installation failed or not found in PATH"
        log_info "You may need to add Homebrew's Python to your PATH"
        log_info "Add this to your ~/.bash_profile or ~/.bashrc:"
        log_info "  export PATH=\"\$(brew --prefix)/bin:\$PATH\""
        exit 1
    fi
    
    # Verify venv can actually create a venv
    local test_venv_dir
    test_venv_dir=$(mktemp -d)
    if ! python3 -m venv "$test_venv_dir" >/dev/null 2>&1; then
        rm -rf "$test_venv_dir" 2>/dev/null || true
        log_error "Python3 venv module cannot create virtual environments"
        log_error "Try: brew install python@3.12"
        exit 1
    fi
    rm -rf "$test_venv_dir"
    log_success "Python3 venv module verified"
    
    if ! command_exists espeak-ng; then
        log_warn "espeak-ng installation may have failed. Piper may not work correctly."
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
    
    # Clear pip cache to avoid dependency conflicts
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
    
    # Use the correct HuggingFace URL structure
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
    if ! command_exists curl; then
        log_error "curl not found. Please install it to download models."
    else
        log_error "Failed to download model files"
    fi
    cd - >/dev/null || true
    return 1
}

# Detect system architecture
detect_arch() {
    local arch
    arch=$(uname -m)
    case "$arch" in
        arm64)
            ARCH="aarch64"
            ;;
        x86_64)
            ARCH="x86_64"
            ;;
        *)
            ARCH="aarch64"  # Default to Apple Silicon
            log_warn "Unknown architecture $arch, defaulting to aarch64"
            ;;
    esac
    log_info "Detected architecture: $ARCH"
}

# Download and install grars binary from GitHub
download_and_install_binary() {
    log_info "Downloading grars binary from GitHub..."
    
    # Ensure bin directory exists
    mkdir -p "$BIN_DIR"
    
    # Detect architecture
    detect_arch
    
    # Construct binary name: grars-1.0.0-macos-aarch64 or grars-1.0.0-macos-x86_64
    BINARY_NAME="grars-${GRARS_VERSION}-macos-${ARCH}"
    
    # Use specific release tag for v1.0.0, or allow override via RELEASE_TAG env var
    RELEASE_TAG="${RELEASE_TAG:-v1.0.0}"
    DOWNLOAD_URL="https://github.com/$GITHUB_REPO/releases/download/$RELEASE_TAG/$BINARY_NAME"
    
    # Download binary
    if download_file "$DOWNLOAD_URL" "$GRARS_BIN"; then
        chmod +x "$GRARS_BIN"
        log_success "Binary downloaded and installed to $GRARS_BIN"
        return 0
    else
        if ! command_exists curl; then
            log_error "curl not found. Please install it."
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

# Main installation function
main() {
    echo "=========================================="
    echo "  grars Installation Script (macOS)"
    echo "=========================================="
    echo ""
    
    # Check if running on macOS
    if [[ "$(uname)" != "Darwin" ]]; then
        log_error "This script is for macOS only"
        log_info "Use install.sh for Linux"
        exit 1
    fi
    
    check_homebrew
    check_and_install_dependencies
    install_binary
    create_venv
    install_piper
    
    # Download model if not present (download_model checks if it exists first)
    echo ""
    download_model
    
    echo ""
    log_success "Installation complete!"
    echo ""
    echo "grars binary: $GRARS_BIN"
    echo "Piper venv: $VENV_DIR/bin/piper"
    echo "Models directory: $MODELS_DIR"
    echo ""
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        log_warn "$HOME/.local/bin is not in your PATH"
        echo "Add this to your ~/.bash_profile or ~/.bashrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
    fi
    echo "Run grars with: grars"
    echo ""
}

# Run main function
main "$@"


#!/bin/bash
# Self-contained uninstall script for insight-reader
# Removes the insight-reader binary, virtual environment, models, and platform-specific files

# Temporarily disable unbound variable checking to safely check BASH_SOURCE
set +u
# Initialize BASH_SOURCE[0] if unbound (happens when script is piped from curl)
# This prevents "unbound variable" errors when 'set -u' is enabled
if [ -z "${BASH_SOURCE[0]:-}" ]; then
    BASH_SOURCE[0]=""
fi
# Now re-enable strict mode
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
INSTALL_DIR="$HOME/.local/share/insight-reader"
BIN_DIR="$HOME/.local/bin"
VENV_DIR="$INSTALL_DIR/venv"
MODELS_DIR="$INSTALL_DIR/models"
INSIGHT_READER_BIN="$BIN_DIR/insight-reader"

# Platform-specific paths
OS=$(uname -s)
if [ "$OS" = "Darwin" ]; then
    APP_BUNDLE="/Applications/insight-reader.app"
    DESKTOP_FILE=""
    ICON_FILE=""
else
    APP_BUNDLE=""
    DESKTOP_FILE="$HOME/.local/share/applications/insight-reader.desktop"
    ICON_FILE="$HOME/.local/share/icons/hicolor/scalable/apps/insight-reader.svg"
fi

CONFIG_FILE="$HOME/.config/insight-reader/config.json"
CONFIG_DIR="$HOME/.config/insight-reader"
LOG_DIR="$HOME/.local/share/insight-reader/logs"
CACHE_INSTALL_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/insight-reader-install"

# Parse arguments
FORCE_PROJECT_ROOT=false
FORCE_USER_DIR=false
AUTO_CONFIRM=false

for arg in "$@"; do
    case "$arg" in
        --project-root)
            FORCE_PROJECT_ROOT=true
            ;;
        --user)
            FORCE_USER_DIR=true
            ;;
        --yes|-y)
            AUTO_CONFIRM=true
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --project-root Force removal from project root only"
            echo "  --user        Force removal from user directory only"
            echo "  --yes, -y     Skip confirmation prompt (for non-interactive use)"
            echo "  --help, -h    Show this help message"
            echo ""
            echo "If no location is specified, auto-detects based on current directory."
            echo "Note: Models are always removed along with the venv."
            echo ""
            echo "Non-interactive usage (e.g., with curl):"
            echo "  curl -fsSL URL | bash -s -- --yes"
            exit 0
            ;;
        *)
            log_error "Unknown option: $arg"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Determine which locations to clean
PROJECT_ROOT_DIR="$(pwd)"
PROJECT_VENV="$PROJECT_ROOT_DIR/venv"
PROJECT_MODELS="$PROJECT_ROOT_DIR/models"
USER_DIR="$INSTALL_DIR"
USER_VENV="$VENV_DIR"
USER_MODELS="$MODELS_DIR"

CLEAN_PROJECT=false
CLEAN_USER=false

if [ "$FORCE_PROJECT_ROOT" = true ]; then
    CLEAN_PROJECT=true
elif [ "$FORCE_USER_DIR" = true ]; then
    CLEAN_USER=true
else
    # Auto-detect: check if we're in project root
    if [ -f "$PROJECT_ROOT_DIR/Cargo.toml" ] && [ -d "$PROJECT_ROOT_DIR/src" ]; then
        CLEAN_PROJECT=true
        log_info "Detected project root, will clean: $PROJECT_ROOT_DIR"
    fi
    
    # Always check user directory too (might have both)
    if [ -d "$USER_DIR" ]; then
        CLEAN_USER=true
        log_info "Found user installation, will clean: $USER_DIR"
    fi
fi

# If nothing detected, ask user
if [ "$CLEAN_PROJECT" = false ] && [ "$CLEAN_USER" = false ]; then
    log_warn "No installation detected in current location or user directory."
    log_info "Checking common locations..."
    
    if [ -d "$PROJECT_VENV" ] || [ -d "$PROJECT_MODELS" ]; then
        CLEAN_PROJECT=true
        log_info "Found installation in project root"
    fi
    
    if [ -d "$USER_VENV" ] || [ -d "$USER_MODELS" ]; then
        CLEAN_USER=true
        log_info "Found installation in user directory"
    fi
    
    if [ "$CLEAN_PROJECT" = false ] && [ "$CLEAN_USER" = false ]; then
        log_error "No insight-reader installation found to remove."
        exit 1
    fi
fi

# Show what will be removed
echo "=========================================="
if [ "$OS" = "Darwin" ]; then
    echo "  Insight Reader Uninstall Script (macOS)"
else
    echo "  Insight Reader Uninstall Script (Linux)"
fi
echo "=========================================="
echo ""

ITEMS_TO_REMOVE=()

if [ "$CLEAN_PROJECT" = true ]; then
    if [ -d "$PROJECT_VENV" ]; then
        ITEMS_TO_REMOVE+=("Project venv: $PROJECT_VENV")
    fi
    if [ -d "$PROJECT_MODELS" ]; then
        ITEMS_TO_REMOVE+=("Project models: $PROJECT_MODELS")
    fi
fi

if [ "$CLEAN_USER" = true ]; then
    if [ -d "$USER_VENV" ]; then
        ITEMS_TO_REMOVE+=("User venv: $USER_VENV")
    fi
    if [ -d "$USER_MODELS" ]; then
        ITEMS_TO_REMOVE+=("User models: $USER_MODELS")
    fi
    if [ -f "$INSIGHT_READER_BIN" ]; then
        ITEMS_TO_REMOVE+=("insight-reader binary: $INSIGHT_READER_BIN")
    fi
    if [ -n "$APP_BUNDLE" ] && [ -d "$APP_BUNDLE" ]; then
        ITEMS_TO_REMOVE+=("App bundle: $APP_BUNDLE")
    fi
    if [ -n "$DESKTOP_FILE" ] && [ -f "$DESKTOP_FILE" ]; then
        ITEMS_TO_REMOVE+=("Desktop file: $DESKTOP_FILE")
    fi
    if [ -n "$ICON_FILE" ] && [ -f "$ICON_FILE" ]; then
        ITEMS_TO_REMOVE+=("Icon: $ICON_FILE")
    fi
    if [ -f "$CONFIG_FILE" ]; then
        ITEMS_TO_REMOVE+=("Config file: $CONFIG_FILE")
    fi
    if [ -d "$LOG_DIR" ]; then
        ITEMS_TO_REMOVE+=("Log directory: $LOG_DIR")
    fi
    if [ -d "$CACHE_INSTALL_DIR" ]; then
        ITEMS_TO_REMOVE+=("Install cache directory: $CACHE_INSTALL_DIR")
    fi
fi

if [ ${#ITEMS_TO_REMOVE[@]} -eq 0 ]; then
    log_warn "Nothing to remove (no matching directories found)"
    exit 0
fi

log_info "The following will be removed:"
for item in "${ITEMS_TO_REMOVE[@]}"; do
    echo "  - $item"
done

echo ""
if [ "$AUTO_CONFIRM" = true ]; then
    log_info "Auto-confirming removal (--yes flag)"
else
    # When script is piped (e.g., curl ... | bash), stdin is the script itself.
    # We need to read from /dev/tty (the terminal) for user input.
    if [ -t 0 ]; then
        # stdin is a terminal, read normally
        read -p "Continue with removal? [y/N] " -n 1 -r
        echo ""
    elif [ -e /dev/tty ]; then
        # stdin is not a terminal (piped), try reading from /dev/tty
        echo -n "Continue with removal? [y/N] "
        read -n 1 -r REPLY </dev/tty
        echo ""
    else
        log_error "Cannot read user input (no terminal available)"
        log_info "Use --yes flag for non-interactive mode:"
        log_info "  curl -fsSL URL | bash -s -- --yes"
        exit 1
    fi
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Cancelled"
        exit 0
    fi
fi

# Remove project root installation
if [ "$CLEAN_PROJECT" = true ]; then
    if [ -d "$PROJECT_VENV" ]; then
        log_info "Removing project venv: $PROJECT_VENV"
        rm -rf "$PROJECT_VENV"
        log_success "Removed project venv"
    else
        log_info "Project venv not found: $PROJECT_VENV"
    fi
    
    if [ -d "$PROJECT_MODELS" ]; then
        log_info "Removing project models: $PROJECT_MODELS"
        rm -rf "$PROJECT_MODELS"
        log_success "Removed project models"
    fi
fi

# Remove user directory installation
if [ "$CLEAN_USER" = true ]; then
    if [ -d "$USER_VENV" ]; then
        log_info "Removing user venv: $USER_VENV"
        rm -rf "$USER_VENV"
        log_success "Removed user venv"
    else
        log_info "User venv not found: $USER_VENV"
    fi
    
    if [ -d "$USER_MODELS" ]; then
        log_info "Removing user models: $USER_MODELS"
        rm -rf "$USER_MODELS"
        log_success "Removed user models"
    fi
    
    # Remove insight-reader binary
    if [ -f "$INSIGHT_READER_BIN" ]; then
        log_info "Removing insight-reader binary: $INSIGHT_READER_BIN"
        rm -f "$INSIGHT_READER_BIN"
        log_success "Removed insight-reader binary"
    fi
    
    # Remove app bundle (macOS-specific)
    if [ -n "$APP_BUNDLE" ] && [ -d "$APP_BUNDLE" ]; then
        log_info "Removing app bundle: $APP_BUNDLE"
        rm -rf "$APP_BUNDLE"
        log_success "Removed app bundle"
    fi
    
    # Remove desktop file (Linux-specific)
    if [ -n "$DESKTOP_FILE" ] && [ -f "$DESKTOP_FILE" ]; then
        log_info "Removing desktop file: $DESKTOP_FILE"
        rm -f "$DESKTOP_FILE"
        log_success "Removed desktop file"
        
        # Update desktop database if available
        if command -v update-desktop-database >/dev/null 2>&1; then
            update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
            log_info "Desktop database updated"
        fi
        # KDE uses kbuildsycoca
        if command -v kbuildsycoca6 >/dev/null 2>&1; then
            kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
            log_info "KDE 6 application database updated"
        elif command -v kbuildsycoca5 >/dev/null 2>&1; then
            kbuildsycoca5 --noincremental >/dev/null 2>&1 || true
            log_info "KDE 5 application database updated"
        fi
    fi
    
    # Remove icon (Linux-specific)
    if [ -n "$ICON_FILE" ] && [ -f "$ICON_FILE" ]; then
        log_info "Removing icon: $ICON_FILE"
        rm -f "$ICON_FILE"
        log_success "Removed icon"
        
        # Update icon cache if available
        if command -v gtk-update-icon-cache >/dev/null 2>&1; then
            gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
            log_info "Icon cache updated"
        fi
        # KDE uses kbuildsycoca for icons too
        if command -v kbuildsycoca6 >/dev/null 2>&1; then
            kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
        elif command -v kbuildsycoca5 >/dev/null 2>&1; then
            kbuildsycoca5 --noincremental >/dev/null 2>&1 || true
        fi
    fi
    
    # Remove config file
    if [ -f "$CONFIG_FILE" ]; then
        log_info "Removing config file: $CONFIG_FILE"
        rm -f "$CONFIG_FILE"
        log_success "Removed config file"
    fi
    
    # Remove config directory if it's empty
    if [ -d "$CONFIG_DIR" ]; then
        if [ -z "$(ls -A "$CONFIG_DIR" 2>/dev/null)" ]; then
            log_info "Removing empty config directory: $CONFIG_DIR"
            rmdir "$CONFIG_DIR"
            log_success "Removed empty config directory"
        fi
    fi
    
    # Remove log directory
    if [ -d "$LOG_DIR" ]; then
        log_info "Removing log directory: $LOG_DIR"
        rm -rf "$LOG_DIR"
        log_success "Removed log directory"
    fi
    
    # Remove user directory (fully remove, not just when empty)
    if [ -d "$USER_DIR" ]; then
        log_info "Removing user directory: $USER_DIR"
        rm -rf "$USER_DIR"
        log_success "Removed user directory"
    fi
    
    # Remove install cache directory
    if [ -d "$CACHE_INSTALL_DIR" ]; then
        log_info "Removing install cache directory: $CACHE_INSTALL_DIR"
        rm -rf "$CACHE_INSTALL_DIR"
        log_success "Removed install cache directory"
    fi
fi

echo ""
log_success "Uninstall complete!"
echo ""
log_info "You can now run ./install.sh to reinstall."

#!/bin/bash
# Uninstall script for grars
# Removes the grars binary, virtual environment, and models
#
# Usage:
#   ./uninstall.sh [--project-root] [--user]
#
# Options:
#   --project-root Force removal from project root only
#   --user        Force removal from user directory only
#   (no options)  Auto-detect and remove from current location

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

# Parse arguments
# Models are always removed (no --all flag needed)
FORCE_PROJECT_ROOT=false
FORCE_USER_DIR=false

for arg in "$@"; do
    case "$arg" in
        --project-root)
            FORCE_PROJECT_ROOT=true
            ;;
        --user)
            FORCE_USER_DIR=true
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --project-root Force removal from project root only"
            echo "  --user        Force removal from user directory only"
            echo "  --help, -h    Show this help message"
            echo ""
            echo "If no location is specified, auto-detects based on current directory."
            echo "Note: Models are always removed along with the venv."
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
USER_DIR="$HOME/.local/share/grars"
USER_VENV="$USER_DIR/venv"
USER_MODELS="$USER_DIR/models"
GRARS_BIN="$HOME/.local/bin/grars"

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
        log_error "No grars installation found to remove."
        exit 1
    fi
fi

# Show what will be removed
echo "=========================================="
echo "  grars Uninstall Script"
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
    if [ -f "$GRARS_BIN" ]; then
        ITEMS_TO_REMOVE+=("grars binary: $GRARS_BIN")
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
read -p "Continue with removal? [y/N] " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    log_info "Cancelled"
    exit 0
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
    
    # Remove grars binary
    if [ -f "$GRARS_BIN" ]; then
        log_info "Removing grars binary: $GRARS_BIN"
        rm -f "$GRARS_BIN"
        log_success "Removed grars binary"
    fi
    
    # Remove user directory if it's empty
    if [ -d "$USER_DIR" ]; then
        if [ -z "$(ls -A "$USER_DIR" 2>/dev/null)" ]; then
            log_info "Removing empty user directory: $USER_DIR"
            rmdir "$USER_DIR"
            log_success "Removed empty user directory"
        fi
    fi
fi

echo ""
log_success "Uninstall complete!"
echo ""
    log_info "You can now run ./install.sh to reinstall."


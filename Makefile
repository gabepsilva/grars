# Makefile for grars
# Builds release binary with version and platform info
# Creates versioned artifacts: grars-1.0.0-linux-x86_64, grars-1.0.0-macos-aarch64, etc.
# Also creates a symlink 'grars' pointing to the versioned binary

BINARY_NAME := grars
RELEASE_DIR := target/release
BUILD_DIR := build

# Detect platform (OS and architecture)
UNAME_S := $(shell uname -s 2>/dev/null || echo "Unknown")
UNAME_M := $(shell uname -m 2>/dev/null || echo "unknown")

# Map OS names (normalize to lowercase)
# Supported: linux, macos, windows, freebsd, openbsd
ifeq ($(shell echo $(UNAME_S) | tr A-Z a-z),linux)
	OS := linux
else ifeq ($(shell echo $(UNAME_S) | tr A-Z a-z),darwin)
	OS := macos
else ifeq ($(shell echo $(UNAME_S) | tr A-Z a-z),mingw32)
	OS := windows
else ifeq ($(shell echo $(UNAME_S) | tr A-Z a-z),mingw64)
	OS := windows
else ifeq ($(shell echo $(UNAME_S) | tr A-Z a-z),msys_nt)
	OS := windows
else ifeq ($(shell echo $(UNAME_S) | tr A-Z a-z),cygwin)
	OS := windows
else ifeq ($(shell echo $(UNAME_S) | tr A-Z a-z),freebsd)
	OS := freebsd
else ifeq ($(shell echo $(UNAME_S) | tr A-Z a-z),openbsd)
	OS := openbsd
else
	# Default to linux if OS cannot be detected
	OS := linux
endif

# Map architecture names
# Supported: x86_64, aarch64, armv7, i686
ifeq ($(UNAME_M),x86_64)
	ARCH := x86_64
else ifeq ($(UNAME_M),amd64)
	ARCH := x86_64
else ifeq ($(UNAME_M),aarch64)
	ARCH := aarch64
else ifeq ($(UNAME_M),arm64)
	ARCH := aarch64
else ifeq ($(UNAME_M),armv7l)
	ARCH := armv7
else ifeq ($(UNAME_M),i386)
	ARCH := i686
else ifeq ($(UNAME_M),i686)
	ARCH := i686
else
	# Default to x86_64 if architecture cannot be detected
	ARCH := x86_64
endif

# Build artifact name with version and platform
# Example: grars-linux-x86_64
ARTIFACT_NAME := $(BINARY_NAME)-$(OS)-$(ARCH)
ARTIFACT_PATH := $(BUILD_DIR)/$(ARTIFACT_NAME)
SYMLINK_PATH := $(BUILD_DIR)/$(BINARY_NAME)

.PHONY: all build clean install help

all: build

# Build release binary and create versioned artifact
# Always runs cargo build --release for native target
# Supports cross-compilation via TARGET environment variable
# Example: TARGET=x86_64-unknown-linux-gnu make build
build:
	@if [ -n "$(TARGET)" ]; then \
		echo "Cross-compiling for target: $(TARGET)"; \
		echo "Installing target if not present..."; \
		rustup target add $(TARGET) 2>/dev/null || true; \
		cargo build --release --target $(TARGET); \
		RELEASE_BINARY=target/$(TARGET)/release/$(BINARY_NAME); \
	else \
		echo "Building release binary for native target..."; \
		cargo build --release; \
		RELEASE_BINARY=$(RELEASE_DIR)/$(BINARY_NAME); \
	fi
	@echo "Creating build directory..."
	@mkdir -p $(BUILD_DIR)
	@echo "Copying binary to $(ARTIFACT_NAME)..."
	@if [ -n "$(TARGET)" ]; then \
		cp target/$(TARGET)/release/$(BINARY_NAME) $(ARTIFACT_PATH); \
	else \
		cp $(RELEASE_DIR)/$(BINARY_NAME) $(ARTIFACT_PATH); \
	fi
	@chmod +x $(ARTIFACT_PATH)
	@echo "✓ Built: $(ARTIFACT_PATH)"

# Create symlink to versioned binary (legacy target, now handled in build)
$(SYMLINK_PATH): $(ARTIFACT_PATH)
	@echo "Creating symlink $(BINARY_NAME) -> $(ARTIFACT_NAME)..."
	@cd $(BUILD_DIR) && ln -sf $(ARTIFACT_NAME) $(BINARY_NAME)
	@echo "✓ Symlink created: $(SYMLINK_PATH)"

# Clean build artifacts
clean:
	@echo "Cleaning build directory..."
	@rm -rf $(BUILD_DIR)
	@echo "Cleaning cargo build artifacts..."
	@cargo clean
	@echo "✓ Cleaned"

# Install to ~/.local/bin (optional)
install: build
	@echo "Installing to ~/.local/bin..."
	@mkdir -p ~/.local/bin
	@cp $(ARTIFACT_PATH) ~/.local/bin/$(BINARY_NAME)
	@chmod +x ~/.local/bin/$(BINARY_NAME)
	@echo "✓ Installed to ~/.local/bin/$(BINARY_NAME)"

# Show help
help:
	@echo "grars Makefile - Build release artifacts with version and platform info"
	@echo ""
	@echo "Usage:"
	@echo "  make build    - Build release binary and create versioned artifact"
	@echo "  make clean    - Remove all build artifacts (build/ and target/)"
	@echo "  make install  - Build and install binary to ~/.local/bin/grars"
	@echo "  make help     - Show this help message"
	@echo ""
	@echo "Build Output:"
	@echo "  Artifact: $(ARTIFACT_NAME)"
	@echo "  Symlink:  $(BINARY_NAME) -> $(ARTIFACT_NAME)"
	@echo "  Location: $(BUILD_DIR)/"
	@echo ""
	@echo "Detected Platform:"
	@echo "  OS:   $(OS)"
	@echo "  Arch: $(ARCH)"
	@echo ""
	@echo "Cross-compilation (optional):"
	@echo "  Set TARGET environment variable to build for different platforms:"
	@echo "    TARGET=x86_64-unknown-linux-gnu make build    # Linux x86_64"
	@echo "    TARGET=aarch64-unknown-linux-gnu make build   # Linux ARM64"
	@echo "    TARGET=x86_64-apple-darwin make build         # macOS x86_64"
	@echo "    TARGET=aarch64-apple-darwin make build        # macOS ARM64 (Apple Silicon)"
	@echo ""
	@echo "  Note: 'unknown' in target triple is Rust's required format for generic Linux"
	@echo ""
	@echo "Cross-compilation Requirements:"
	@echo "  1. Install target: rustup target add <target>"
	@echo "  2. Cross-compilation toolchain (gcc, linker, etc. for target platform)"
	@echo "  3. Note: Some native dependencies may require additional setup"


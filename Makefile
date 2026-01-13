# Makefile for insight-reader
# Builds release binary to target/release/

BINARY_NAME := insight-reader
RELEASE_DIR := target/release
BINARY_PATH := $(RELEASE_DIR)/$(BINARY_NAME)

.PHONY: all build clean install help copy-to-vms

all: build

# Build release binary
# Supports cross-compilation via TARGET environment variable
# Example: TARGET=x86_64-unknown-linux-gnu make build
build:
	@if [ -n "$(TARGET)" ]; then \
		echo "Cross-compiling for target: $(TARGET)"; \
		rustup target add $(TARGET) 2>/dev/null || true; \
		cargo build --release --target $(TARGET); \
		echo "✓ Built: target/$(TARGET)/release/$(BINARY_NAME)"; \
	else \
		echo "Building release binary..."; \
		cargo build --release; \
		echo "✓ Built: $(BINARY_PATH)"; \
	fi

# Clean build artifacts
clean:
	@echo "Cleaning cargo build artifacts..."
	@cargo clean
	@echo "✓ Cleaned"

# Install to ~/.local/bin (optional)
install: build
	@echo "Installing to ~/.local/bin..."
	@mkdir -p ~/.local/bin
	@cp $(BINARY_PATH) ~/.local/bin/$(BINARY_NAME)
	@chmod +x ~/.local/bin/$(BINARY_NAME)
	@echo "✓ Installed to ~/.local/bin/$(BINARY_NAME)"

# Copy release binary to all test VMs (continues even if some VMs are offline)
# Copies from target/release/insight-reader
copy-to-vms:
	@if [ ! -f "target/release/$(BINARY_NAME)" ]; then \
		echo "Release binary not found. Building..."; \
		$(MAKE) build; \
	fi
	@echo "Copying release binary from target/release/$(BINARY_NAME) to all test VMs..."
	@echo ""
	@echo "Copying to Ubuntu Desktop Minimal (port 2222)..."
	@-scp -P 2222 target/release/$(BINARY_NAME) vboxuser@localhost:/tmp/$(BINARY_NAME) 2>/dev/null && \
		ssh -p 2222 vboxuser@localhost "cp /tmp/$(BINARY_NAME) ~/.local/bin/$(BINARY_NAME) && chmod +x ~/.local/bin/$(BINARY_NAME)" 2>/dev/null && \
		echo "✓ Copied to Ubuntu VM" || echo "✗ Ubuntu VM offline or unreachable"
	@echo ""
	@echo "Copying to Manjaro KDE (port 2223)..."
	@-scp -P 2223 target/release/$(BINARY_NAME) vboxuser@localhost:/tmp/$(BINARY_NAME) 2>/dev/null && \
		ssh -p 2223 vboxuser@localhost "cp /tmp/$(BINARY_NAME) ~/.local/bin/$(BINARY_NAME) && chmod +x ~/.local/bin/$(BINARY_NAME)" 2>/dev/null && \
		echo "✓ Copied to Manjaro VM" || echo "✗ Manjaro VM offline or unreachable"
	@echo ""
	@echo "Copying to Fedora 43 Workstation (port 2224)..."
	@-scp -P 2224 target/release/$(BINARY_NAME) vboxuser@localhost:/tmp/$(BINARY_NAME) 2>/dev/null && \
		ssh -p 2224 vboxuser@localhost "cp /tmp/$(BINARY_NAME) ~/.local/bin/$(BINARY_NAME) && chmod +x ~/.local/bin/$(BINARY_NAME)" 2>/dev/null && \
		echo "✓ Copied to Fedora VM" || echo "✗ Fedora VM offline or unreachable"
	@echo ""
	@echo "Copying to PopOs Cosmic (port 2225)..."
	@-scp -P 2225 target/release/$(BINARY_NAME) vboxuser@localhost:/tmp/$(BINARY_NAME) 2>/dev/null && \
		ssh -p 2225 vboxuser@localhost "cp /tmp/$(BINARY_NAME) ~/.local/bin/$(BINARY_NAME) && chmod +x ~/.local/bin/$(BINARY_NAME)" 2>/dev/null && \
		echo "✓ Copied to PopOs VM" || echo "✗ PopOs VM offline or unreachable"
	@echo ""
	@echo "✓ Copy process completed (some VMs may be offline)"

# Show help
help:
	@echo "insight-reader Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make build       - Build release binary to target/release/"
	@echo "  make clean       - Remove all build artifacts (cargo clean)"
	@echo "  make install     - Build and install binary to ~/.local/bin/insight-reader"
	@echo "  make copy-to-vms - Build release binary and copy to all test VMs"
	@echo "  make help        - Show this help message"
	@echo ""
	@echo "Build Output:"
	@echo "  Binary: $(BINARY_PATH)"
	@echo ""
	@echo "Cross-compilation (optional):"
	@echo "  Set TARGET environment variable to build for different platforms:"
	@echo "    TARGET=x86_64-unknown-linux-gnu make build    # Linux x86_64"
	@echo "    TARGET=aarch64-unknown-linux-gnu make build   # Linux ARM64"
	@echo "    TARGET=x86_64-apple-darwin make build         # macOS x86_64"
	@echo "    TARGET=aarch64-apple-darwin make build        # macOS ARM64 (Apple Silicon)"
	@echo ""
	@echo "Cross-compilation Requirements:"
	@echo "  1. Install target: rustup target add <target>"
	@echo "  2. Cross-compilation toolchain (gcc, linker, etc. for target platform)"


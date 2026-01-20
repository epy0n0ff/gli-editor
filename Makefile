.PHONY: help build release test clean install uninstall clippy fmt check run dev dist dist-clean

# Variables
VERSION := $(shell grep '^version' Cargo.toml | head -n1 | cut -d'"' -f2)
DIST_DIR := dist
TARGET := $(shell rustc -vV | grep '^host:' | cut -d' ' -f2)

# Detect OS
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    OS_NAME := linux
    ARCHIVE_EXT := tar.gz
endif
ifeq ($(UNAME_S),Darwin)
    OS_NAME := macos
    ARCHIVE_EXT := tar.gz
endif
ifeq ($(OS),Windows_NT)
    OS_NAME := windows
    ARCHIVE_EXT := zip
endif

# Detect architecture
UNAME_M := $(shell uname -m)
ifeq ($(UNAME_M),x86_64)
    ARCH := amd64
endif
ifeq ($(UNAME_M),arm64)
    ARCH := arm64
endif
ifeq ($(UNAME_M),aarch64)
    ARCH := arm64
endif

BINARY_NAME := gli-editor
ARCHIVE_NAME := $(BINARY_NAME)-$(OS_NAME)-$(ARCH)

# Default target
help:
	@echo "Available targets:"
	@echo "  make build        - Build the project in debug mode"
	@echo "  make release      - Build the project in release mode"
	@echo "  make test         - Run all tests"
	@echo "  make clippy       - Run clippy linter"
	@echo "  make fmt          - Format code with rustfmt"
	@echo "  make check        - Run all checks (fmt, clippy, test)"
	@echo "  make clean        - Remove build artifacts"
	@echo "  make install      - Install the binary to ~/.cargo/bin"
	@echo "  make uninstall    - Uninstall the binary from ~/.cargo/bin"
	@echo "  make run          - Run the application in debug mode"
	@echo "  make dev          - Run the application with a test file"
	@echo "  make dist         - Create distribution archive for release"
	@echo "  make dist-clean   - Remove distribution directory"

# Build targets
build:
	cargo build

release:
	cargo build --release

# Test targets
test:
	cargo test

# Lint and format targets
clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

# Combined check target
check: fmt-check clippy test
	@echo "All checks passed!"

# Clean target
clean:
	cargo clean

# Install/uninstall targets
install: release
	cargo install --path .

uninstall:
	cargo uninstall gli-editor

# Run targets
run:
	cargo run

dev:
	cargo run -- -f .gitleaksignore

# Watch mode (requires cargo-watch: cargo install cargo-watch)
watch:
	cargo watch -x 'run -- -f .gitleaksignore'

# Distribution targets
dist: release
	@echo "Creating distribution archive: $(ARCHIVE_NAME).$(ARCHIVE_EXT)"
	@mkdir -p $(DIST_DIR)
ifeq ($(OS_NAME),windows)
	@cd target/release && 7z a ../../$(DIST_DIR)/$(ARCHIVE_NAME).zip $(BINARY_NAME).exe
else
	@strip target/release/$(BINARY_NAME)
	@cd target/release && tar czf ../../$(DIST_DIR)/$(ARCHIVE_NAME).tar.gz $(BINARY_NAME)
endif
	@echo "Distribution archive created: $(DIST_DIR)/$(ARCHIVE_NAME).$(ARCHIVE_EXT)"
	@ls -lh $(DIST_DIR)/$(ARCHIVE_NAME).$(ARCHIVE_EXT)

dist-clean:
	@rm -rf $(DIST_DIR)
	@echo "Distribution directory removed"

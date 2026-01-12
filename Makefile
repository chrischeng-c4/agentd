# Specter Makefile
.PHONY: help build install test clean dev run release check fmt lint docs

# Default target
help:
	@echo "Specter - Development Commands"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build      - Build debug binary"
	@echo "  release    - Build optimized release binary"
	@echo "  install    - Install specter to ~/.cargo/bin"
	@echo "  dev        - Build and run in dev mode"
	@echo "  test       - Run all tests"
	@echo "  check      - Quick compile check"
	@echo "  fmt        - Format code"
	@echo "  lint       - Run clippy linter"
	@echo "  clean      - Remove build artifacts"
	@echo "  docs       - Generate documentation"
	@echo "  run        - Run specter with args (make run ARGS='--help')"

# Build debug binary
build:
	cargo build

# Build release binary (optimized)
release:
	cargo build --release
	@echo ""
	@echo "Release binary: target/release/specter"
	@ls -lh target/release/specter

# Install to system
install:
	cargo install --path . --force
	@echo ""
	@echo "Installed to: ~/.cargo/bin/specter"
	@specter --version

# Development mode (watch and rebuild)
dev:
	cargo watch -x 'build --release' -x 'test'

# Run specter
run:
	cargo run -- $(ARGS)

# Run tests
test:
	cargo test

# Quick compile check
check:
	cargo check --all-targets --all-features

# Format code
fmt:
	cargo fmt --all

# Run clippy
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/

# Generate documentation
docs:
	cargo doc --no-deps --open

# Build for multiple platforms (requires cross)
build-all:
	@echo "Building for multiple platforms..."
	cargo build --release --target x86_64-unknown-linux-gnu
	cargo build --release --target aarch64-unknown-linux-gnu
	cargo build --release --target x86_64-apple-darwin
	cargo build --release --target aarch64-apple-darwin

# Run integration tests
test-integration:
	@echo "Running integration tests..."
	cd test-project && ../target/release/specter init
	cd test-project && ../target/release/specter list

# Install dev dependencies
setup-dev:
	rustup component add rustfmt clippy
	cargo install cargo-watch

# Show version info
version:
	@cargo pkgid | cut -d# -f2
	@rustc --version
	@cargo --version

# Benchmark (if we add benchmarks)
bench:
	cargo bench

# Security audit
audit:
	cargo audit

# Update dependencies
update:
	cargo update
	@echo "Updated dependencies. Review Cargo.lock changes."

# Create a new release (tags and builds)
tag-release:
	@echo "Current version: $$(cargo pkgid | cut -d# -f2)"
	@echo "Enter new version (e.g., 0.2.0):"
	@read VERSION && \
		sed -i.bak "s/^version = .*/version = \"$$VERSION\"/" Cargo.toml && \
		git add Cargo.toml && \
		git commit -m "chore: bump version to $$VERSION" && \
		git tag -a "v$$VERSION" -m "Release v$$VERSION" && \
		echo "Tagged v$$VERSION. Push with: git push && git push --tags"

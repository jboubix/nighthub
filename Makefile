# Nighthub - GitHub Actions Terminal Monitor Makefile

.PHONY: help build release test clean install fmt clippy run dev package

# Default target
help:
	@echo "Nighthub - GitHub Actions Terminal Monitor"
	@echo ""
	@echo "Available targets:"
	@echo "  build     - Build the project in debug mode"
	@echo "  release   - Build optimized release binary"
	@echo "  test      - Run all tests"
	@echo "  clean     - Clean build artifacts"
	@echo "  install   - Install the release binary to /usr/local/bin"
	@echo "  fmt       - Format code with rustfmt"
	@echo "  clippy    - Run clippy lints"
	@echo "  run       - Run the application in debug mode"
	@echo "  dev       - Run with development environment"
	@echo "  package   - Create distribution package"
	@echo "  help      - Show this help message"

# Build in debug mode
build:
	cargo build

# Build optimized release binary
release:
	cargo build --release
	strip target/release/nighthub
	cp target/release/nighthub nighthub

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean
	rm -f nighthub
	rm -f *.tar.gz

# Install to system (atomic replacement - safe while running)
install: release
	mkdir -p ~/.local/bin
	mv nighthub ~/.local/bin/

# Format code
fmt:
	cargo fmt

# Run clippy lints
clippy:
	cargo clippy

# Run in debug mode
run: build
	./target/debug/nighthub

# Run with development environment
dev: build
	@if [ -z "$(GITHUB_TOKEN)" ]; then \
		echo "Warning: GITHUB_TOKEN not set. Set it with: export GITHUB_TOKEN=your_token"; \
	fi
	@if [ -z "$(REPOS)" ]; then \
		echo "Warning: REPOS not set. Set it with: export REPOS=owner1/repo1,owner2/repo2"; \
	fi
	./target/debug/nighthub

# Create distribution package
package: release
	tar -czf nighthub-linux.tar.gz nighthub README.md
	@echo "Created package: nighthub-linux.tar.gz"

# Development workflow
dev-check: fmt clippy test
	@echo "All development checks passed!"
# Project variables
BIN_NAME := global_price_index
FRONTEND_DIR := frontend

# Commands
CARGO := cargo
NPM := npm
TSC := npx tsc

# Build all (both Rust and TypeScript)
build: build-rust build-ts
	@echo "Build completed for both Rust and TypeScript"

# Build only Rust
build-rust:
	@echo "Building Rust project..."
	@$(CARGO) build --release

# Build only TypeScript
build-ts:
	@echo "Building TypeScript..."
	@cd $(FRONTEND_DIR) && $(NPM) run build

# Run all (both Rust and TypeScript)
run: run-rust run-ts
	@echo "Running both Rust and TypeScript applications"

# Run only Rust
run-rust:
	@echo "Starting Rust application..."
	@$(CARGO) run --release

# Run only TypeScript
run-ts:
	@echo "Starting TypeScript application..."
	@cd $(FRONTEND_DIR) && $(NPM) start

# Format all code
format: format-rust format-ts
	@echo "Formatting completed for both Rust and TypeScript"

# Format Rust code
format-rust:
	@echo "Formatting Rust..."
	@$(CARGO) fmt

# Format TypeScript code
format-ts:
	@echo "Formatting TypeScript..."
	@cd $(FRONTEND_DIR) && $(NPM) run format

# Lint all code
lint: lint-rust lint-ts
	@echo "Linting completed for both Rust and TypeScript"

# Lint Rust code
lint-rust:
	@echo "Linting Rust..."
	@$(CARGO) clippy --all-targets --allow-features -- -D warnings

# Lint TypeScript code
lint-ts:
	@echo "Linting TypeScript..."
	@cd $(FRONTEND_DIR) && $(NPM) run lint

# Run all tests
test: test-rust test-ts
	@echo "All tests completed"

# Run Rust tests
test-rust:
	@echo "Running Rust tests..."
	@$(CARGO) test

# Run TypeScript tests
test-ts:
	@echo "Running TypeScript tests..."
	@cd $(FRONTEND_DIR) && $(NPM) run test

# Clean all build artifacts
clean: clean-rust clean-ts
	@echo "All build artifacts cleaned"

# Clean Rust build artifacts
clean-rust:
	@echo "Cleaning Rust build artifacts..."
	@$(CARGO) clean

# Clean TypeScript build artifacts
clean-ts:
	@echo "Cleaning TypeScript build artifacts..."
	@cd $(FRONTEND_DIR) && rm -rf node_modules
	@cd $(FRONTEND_DIR) && rm -rf static/js/*.js

# Watch for changes
watch: watch-rust watch-ts
	@echo "Watching for changes in both Rust and TypeScript"

# Watch Rust changes
watch-rust:
	@echo "Watching Rust changes..."
	@$(CARGO) watch -x build

# Watch TypeScript changes
watch-ts:
	@echo "Watching TypeScript changes..."
	@cd $(FRONTEND_DIR) && $(NPM) run watch

# Install all dependencies
install: install-rust install-ts
	@echo "All dependencies installed"

# Install Rust dependencies
install-rust:
	@echo "Installing Rust dependencies..."
	@$(CARGO) build

# Install TypeScript dependencies
install-ts:
	@echo "Installing TypeScript dependencies..."
	@cd $(FRONTEND_DIR) && $(NPM) install

# Help command
help:
	@echo "Available commands:"
	@echo "  make install      - Install all dependencies"
	@echo "  make install-rust - Install Rust dependencies"
	@echo "  make install-ts   - Install TypeScript dependencies"
	@echo "  make build        - Build both Rust and TypeScript"
	@echo "  make build-rust   - Build only Rust"
	@echo "  make build-ts     - Build only TypeScript"
	@echo "  make run          - Run both applications"
	@echo "  make run-rust     - Run only Rust application"
	@echo "  make run-ts       - Run only TypeScript application"
	@echo "  make format       - Format all code"
	@echo "  make format-rust  - Format only Rust code"
	@echo "  make format-ts    - Format only TypeScript code"
	@echo "  make lint         - Lint all code"
	@echo "  make lint-rust    - Lint only Rust code"
	@echo "  make lint-ts      - Lint only TypeScript code"
	@echo "  make test         - Run all tests"
	@echo "  make test-rust    - Run only Rust tests"
	@echo "  make test-ts      - Run only TypeScript tests"
	@echo "  make clean        - Clean all build artifacts"
	@echo "  make clean-rust   - Clean only Rust build artifacts"
	@echo "  make clean-ts     - Clean only TypeScript build artifacts"
	@echo "  make watch        - Watch for changes in both"
	@echo "  make watch-rust   - Watch only Rust changes"
	@echo "  make watch-ts     - Watch only TypeScript changes"
	@echo "  make help         - Show this help message"

.PHONY: all build build-rust build-ts run run-rust run-ts format format-rust format-ts \
        lint lint-rust lint-ts test test-rust test-ts clean clean-rust clean-ts \
        watch watch-rust watch-ts install install-rust install-ts help

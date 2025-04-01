# Project variables
BIN_NAME := global_price_index

# Commands
CARGO := cargo
NPM := npm
TSC := npx tsc
FRONTEND_DIR := frontend

# Build the project
build:
	@echo "Building TypeScript..."
	@cd $(FRONTEND_DIR) && $(NPM) run build
	@echo "Building Rust project..."
	@$(CARGO) build --release

# Run the project
run:
	@echo "Starting the application..."
	@$(CARGO) run --release

# Format the project
format:
	@echo "Formatting TypeScript..."
	@cd $(FRONTEND_DIR) && $(NPM) run format
	@echo "Formatting Rust..."
	@$(CARGO) fmt

# Check for linting
lint:
	@echo "Linting TypeScript..."
	@cd $(FRONTEND_DIR) && $(NPM) run lint
	@echo "Linting Rust..."
	@$(CARGO) clippy --all-targets --allow-features -- -D warnings

# Run tests
test:
	@echo "Running TypeScript tests..."
	@cd $(FRONTEND_DIR) && $(NPM) run test
	@echo "Running Rust tests..."
	@$(CARGO) test

# Clean the project
clean:
	@echo "Cleaning build artifacts..."
	@cd $(FRONTEND_DIR) && rm -rf node_modules
	@cd $(FRONTEND_DIR) && rm -rf static/js/*.js
	@$(CARGO) clean

# Watch for changes and rebuild
watch:
	@echo "Watching for changes..."
	@cd $(FRONTEND_DIR) && $(NPM) run watch

.PHONY: all build clean install run format lint test watch help

# Default target
all: install build

# Install dependencies
install:
	@echo "Installing dependencies..."
	@cd $(FRONTEND_DIR) && $(NPM) install
	@$(CARGO) build

# Help command
help:
	@echo "Available commands:"
	@echo "  make install    - Install npm and cargo dependencies"
	@echo "  make build      - Build the project (TypeScript and Rust)"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make run        - Run the application"
	@echo "  make format     - Format TypeScript and Rust code"
	@echo "  make lint       - Lint TypeScript and Rust code"
	@echo "  make test       - Run TypeScript and Rust tests"
	@echo "  make watch      - Watch for changes and rebuild"
	@echo "  make help       - Show this help message"

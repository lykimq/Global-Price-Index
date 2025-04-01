# Project variables
BIN_NAME := global_price_index

# Commands
CARGO := cargo

# Build the project
build:
	$(CARGO) build --release

# Run the project
run:
	$(CARGO) run --release

# Format the project
format:
	$(CARGO) fmt

# Check for linting
lint:
	$(CARGO) clippy --all-targets --allow-features -- -D warnings

# Run tests
test:
	$(CARGO) test

# Clean the project
clean:
	$(CARGO) clean

.PHONY: all build clean install run

# Default target
all: install build

# Install dependencies
install:
	@echo "Installing dependencies..."
	@npm install
	@cargo build

# Build the project
build:
	@echo "Building Rust project..."
	@cargo build --release

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf node_modules
	@cargo clean

# Run the application
run:
	@echo "Starting the application..."
	@cargo run

# Help command
help:
	@echo "Available commands:"
	@echo "  make install    - Install npm and cargo dependencies"
	@echo "  make build      - Build the project"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make run        - Run the application"
	@echo "  make help       - Show this help message"

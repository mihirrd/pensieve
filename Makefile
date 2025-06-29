# Makefile for Rust project

# Set default log level (can be overridden in shell)
RUST_LOG ?= info

# Default target
all: run

# Run the project with RUST_LOG
run:
	RUST_LOG=$(RUST_LOG) cargo run

docker:
	docker-compose up

# Build the project in debug mode
build:
	cargo build

# Build the project in release mode
release:
	cargo build --release

# Run tests
test:
	cargo test

# Format the code
fmt:
	cargo fmt

# Lint the code
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
clean:
	cargo clean

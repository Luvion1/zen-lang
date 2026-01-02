# Zen Programming Language Makefile

.PHONY: build test clean install run-examples check fmt clippy help ci debug debug-build debug-run fix

# Default target
all: build test

# Build the project
build:
	@echo "Building Zen compiler..."
	cargo build --release

# Run all tests
test:
	@echo "Running tests..."
	cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -f examples/*.ll examples/*/
	find examples -type f -executable -delete

# Install globally
install: build
	@echo "Installing Zen compiler..."
	sudo cp target/release/zen /usr/local/bin/
	@echo "Zen installed to /usr/local/bin/zen"

# Run all examples to verify they compile
run-examples: build
	@echo "Testing all examples..."
	@success=0; total=0; \
	for file in examples/*.zen; do \
		total=$$((total+1)); \
		if ./target/release/zen compile "$$file" >/dev/null 2>&1; then \
			success=$$((success+1)); \
			echo "✅ $$(basename "$$file")"; \
		else \
			echo "❌ $$(basename "$$file")"; \
		fi; \
	done; \
	echo ""; \
	echo "SUCCESS: $$success/$$total files compiled successfully"

# Quick development checks
check: fmt clippy test

# CI/CD pipeline
ci: check run-examples
	@echo "✅ All CI checks passed!"

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt

# Run clippy lints
clippy:
	@echo "Running clippy..."
	cargo clippy -- -D warnings

# Development build (debug)
dev:
	@echo "Building debug version..."
	cargo build

# Debug build with full symbols and no optimization
debug-build:
	@echo "Building debug version with full symbols..."
	RUST_BACKTRACE=1 cargo build --profile dev

# Debug run with backtrace and verbose output
debug-run: debug-build
	@echo "Running debug version with backtrace..."
	RUST_BACKTRACE=full RUST_LOG=debug ./target/debug/zen

# Debug a specific example with full output
debug-%: debug-build
	@echo "Debug running example: $*.zen"
	RUST_BACKTRACE=full RUST_LOG=debug ./target/debug/zen run examples/$*.zen

# Fix warnings and format code
fix:
	@echo "Fixing code issues..."
	cargo fix --allow-dirty --allow-staged
	cargo fmt
	@echo "✅ Code fixed and formatted"

# Run a specific example
run-%: build
	@echo "Running example: $*.zen"
	./target/release/zen run examples/$*.zen

# Compile a specific example
compile-%: build
	@echo "Compiling example: $*.zen"
	./target/release/zen compile examples/$*.zen

# Benchmark compilation speed
benchmark: build
	@echo "Benchmarking compilation speed..."
	@time -p make run-examples >/dev/null

# Show help
help:
	@echo "Zen Programming Language - Available targets:"
	@echo ""
	@echo "  build         - Build release version"
	@echo "  dev           - Build debug version"
	@echo "  debug-build   - Build debug with full symbols"
	@echo "  debug-run     - Run debug version with backtrace"
	@echo "  debug-<name>  - Debug run specific example"
	@echo "  test          - Run all tests"
	@echo "  clean         - Clean build artifacts"
	@echo "  install       - Install globally to /usr/local/bin"
	@echo "  run-examples  - Test compile all examples"
	@echo "  check         - Run fmt + clippy + test"
	@echo "  ci            - Full CI pipeline (check + run-examples)"
	@echo "  fmt           - Format code"
	@echo "  clippy        - Run lints"
	@echo "  fix           - Fix warnings and format code"
	@echo "  benchmark     - Benchmark compilation speed"
	@echo "  run-<name>    - Run specific example (e.g., make run-hello)"
	@echo "  compile-<name> - Compile specific example"
	@echo "  help          - Show this help"

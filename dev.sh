#!/bin/bash

# Zen Language Development Scripts
# This script provides common development tasks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install development dependencies
install_deps() {
    log_info "Installing development dependencies..."
    
    # Install Rust tools
    if command_exists cargo; then
        cargo install cargo-audit
        cargo install cargo-deny
        cargo install cargo-outdated
        cargo install cargo-tree
        cargo install hyperfine
        log_success "Rust development tools installed"
    else
        log_error "Cargo not found. Please install Rust first."
        exit 1
    fi
    
    # Install LLVM if not present
    if ! command_exists llc; then
        log_warning "LLVM not found. Please install LLVM manually."
        echo "Ubuntu/Debian: sudo apt-get install llvm-14 llvm-14-dev"
        echo "macOS: brew install llvm"
        echo "Windows: choco install llvm"
    fi
}

# Run all tests
test_all() {
    log_info "Running comprehensive test suite..."
    
    # Unit tests
    log_info "Running unit tests..."
    cargo test --verbose
    
    # Integration tests
    log_info "Running integration tests..."
    cargo test --test '*' --verbose
    
    # Example tests
    log_info "Testing examples..."
    cargo build --release
    
    for example in examples/*.zen; do
        if [ -f "$example" ]; then
            log_info "Testing $(basename "$example")..."
            timeout 30s ./target/release/zen run "$example" || log_warning "Example $(basename "$example") timed out or failed"
        fi
    done
    
    log_success "All tests completed"
}

# Code quality checks
quality_check() {
    log_info "Running code quality checks..."
    
    # Format check
    log_info "Checking code formatting..."
    cargo fmt --all -- --check
    
    # Clippy
    log_info "Running clippy..."
    cargo clippy --all-targets --all-features -- -D warnings
    
    # Security audit
    log_info "Running security audit..."
    cargo audit
    
    # Dependency check
    log_info "Checking dependencies..."
    cargo deny check
    
    # Documentation check
    log_info "Checking documentation..."
    cargo doc --no-deps --document-private-items
    
    log_success "All quality checks passed"
}

# Performance benchmarks
benchmark() {
    log_info "Running performance benchmarks..."
    
    if ! command_exists hyperfine; then
        log_error "hyperfine not found. Install with: cargo install hyperfine"
        exit 1
    fi
    
    cargo build --release
    
    log_info "Benchmarking compilation speed..."
    hyperfine --warmup 3 --runs 10 \
        './target/release/zen compile examples/hello.zen' \
        './target/release/zen compile examples/arithmetic.zen' \
        './target/release/zen compile examples/functions_advanced.zen'
    
    log_info "Benchmarking execution speed..."
    ./target/release/zen compile examples/arithmetic.zen
    ./target/release/zen compile examples/algorithms.zen
    
    hyperfine --warmup 3 --runs 10 \
        './examples/arithmetic' \
        './examples/algorithms'
    
    log_success "Benchmarks completed"
}

# Build release binaries
build_release() {
    log_info "Building release binaries..."
    
    # Clean previous builds
    cargo clean
    
    # Build for current platform
    cargo build --release
    
    # Test the binary
    ./target/release/zen --help
    ./target/release/zen run examples/hello.zen
    
    log_success "Release binary built and tested"
}

# Generate documentation
generate_docs() {
    log_info "Generating documentation..."
    
    # API documentation
    cargo doc --no-deps --all-features --document-private-items
    
    # Create documentation site
    mkdir -p docs-site
    cp -r target/doc docs-site/api
    cp README.md docs-site/
    cp -r examples docs-site/
    
    log_success "Documentation generated in docs-site/"
}

# Clean build artifacts
clean() {
    log_info "Cleaning build artifacts..."
    
    cargo clean
    rm -rf docs-site/
    rm -f examples/*.o examples/zen_temp_*
    
    # Clean compiled examples
    for example in examples/*.zen; do
        if [ -f "$example" ]; then
            binary_name=$(basename "$example" .zen)
            rm -f "examples/$binary_name"
        fi
    done
    
    log_success "Cleaned all build artifacts"
}

# Development setup
setup() {
    log_info "Setting up development environment..."
    
    # Install dependencies
    install_deps
    
    # Initial build
    cargo build
    
    # Run tests to verify setup
    cargo test
    
    log_success "Development environment setup complete"
}

# CI simulation
ci_local() {
    log_info "Running local CI simulation..."
    
    # Quality checks
    quality_check
    
    # Tests
    test_all
    
    # Build
    build_release
    
    log_success "Local CI simulation completed successfully"
}

# Show help
show_help() {
    echo "Zen Language Development Script"
    echo ""
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  setup        - Set up development environment"
    echo "  test         - Run all tests"
    echo "  quality      - Run code quality checks"
    echo "  benchmark    - Run performance benchmarks"
    echo "  build        - Build release binary"
    echo "  docs         - Generate documentation"
    echo "  clean        - Clean build artifacts"
    echo "  ci           - Run local CI simulation"
    echo "  deps         - Install development dependencies"
    echo "  help         - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 setup     # Initial development setup"
    echo "  $0 test      # Run comprehensive tests"
    echo "  $0 ci        # Simulate CI pipeline locally"
}

# Main script logic
case "${1:-help}" in
    setup)
        setup
        ;;
    test)
        test_all
        ;;
    quality)
        quality_check
        ;;
    benchmark)
        benchmark
        ;;
    build)
        build_release
        ;;
    docs)
        generate_docs
        ;;
    clean)
        clean
        ;;
    ci)
        ci_local
        ;;
    deps)
        install_deps
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        log_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac

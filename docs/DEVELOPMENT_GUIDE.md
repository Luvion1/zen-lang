# Zen Development Guide

Comprehensive guide for developing and contributing to the Zen compiler.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Build System](#build-system)
- [Testing](#testing)
- [Debugging](#debugging)
- [Performance Profiling](#performance-profiling)
- [Release Process](#release-process)
- [Common Tasks](#common-tasks)

---

## Getting Started

### Prerequisites

```bash
# Install Rust 1.70+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install LLVM toolchain
# Ubuntu/Debian
sudo apt-get install llvm lld clang

# macOS
brew install llvm lld

# Windows
# Download from https://releases.llvm.org/
```

### Clone Repository

```bash
git clone https://github.com/Luvion1/zen-lang.git
cd zen-lang
```

### Initial Build

```bash
# Debug build (fast)
cargo build

# Release build (optimized)
cargo build --release
```

---

## Development Environment

### IDE Setup

#### VS Code

Install extensions:
- rust-analyzer (Rust language server)
- CodeLLDB (debugger)
- Even Better TOML (Cargo.toml support)

Workspace settings (`.vscode/settings.json`):
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.loadOutDirsFromCheck": true,
    "rust-analyzer.inlayHints.typeHints.enable": true,
    "rust-analyzer.inlayHints.parameterHints.enable": true
}
```

#### IntelliJ IDEA

- Install Rust plugin
- Enable "External Linter" for clippy

### Development Tools

```bash
# Install useful tools
cargo install cargo-edit       # cargo add, cargo upgrade
cargo install cargo-watch      # cargo watch -x test
cargo install cargo-expand      # macro expansion
cargo install cargo-tree        # dependency tree
cargo install cargo-outdated    # check for updates

# Install linting and formatting tools
rustup component add rustfmt clippy
```

### Git Configuration

```bash
# Set up Git hooks (optional)
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
cargo fmt --check
cargo clippy -- -D warnings
EOF
chmod +x .git/hooks/pre-commit
```

---

## Project Structure

```
zen-lang/
├── .github/
│   └── workflows/          # CI/CD configurations
│       ├── ci.yml
│       ├── build.yml
│       └── examples-test.yml
├── docs/                   # Documentation
│   ├── ARCHITECTURE.md
│   ├── CONTRIBUTING.md
│   ├── LANGUAGE_REFERENCE.md
│   ├── ROADMAP.md
│   └── DEVELOPMENT_GUIDE.md
├── examples/               # Example programs
│   ├── hello.zen
│   ├── match_simple.zen
│   └── ...
├── src/
│   ├── ast/               # AST definitions
│   │   ├── ast.rs
│   │   ├── expr.rs
│   │   ├── mod.rs
│   │   ├── program.rs
│   │   └── statement.rs
│   ├── codegen/           # Code generation
│   │   ├── codegen.rs
│   │   └── mod.rs
│   ├── compiler.rs        # Compiler orchestration
│   ├── lexer/             # Lexical analysis
│   │   ├── lexer.rs
│   │   └── mod.rs
│   ├── parser/            # Parsing
│   │   ├── parser.rs
│   │   └── mod.rs
│   ├── typechecker/       # Type checking
│   │   ├── typechecker.rs
│   │   └── mod.rs
│   ├── cli.rs             # CLI interface
│   ├── main.rs            # Entry point
│   └── lib.rs             # Library exports
├── tests/                 # Integration tests
│   ├── lexer_tests.rs
│   ├── parser_tests.rs
│   └── integration_tests.rs
├── Cargo.toml             # Rust package config
├── Cargo.lock             # Dependency lock file
├── README.md              # Project README
└── ZEN_LANGUAGE.md        # Language documentation
```

---

## Build System

### Cargo Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build
cargo check                    # Quick check without building

# Run
cargo run -- compile test.zen  # Run with arguments
cargo test                     # Run tests
cargo doc --open               # Generate and open docs

# Format
cargo fmt                      # Format code
cargo fmt --check              # Check formatting

# Lint
cargo clippy                   # Run clippy
cargo clippy -- -D warnings    # Treat warnings as errors

# Clean
cargo clean                    # Clean build artifacts
cargo clean --release          # Clean release artifacts
```

### Build Profiles

```toml
# Cargo.toml - Build profiles
[profile.dev]
opt-level = 0          # No optimization (fast builds)

[profile.release]
opt-level = 3          # Maximum optimization
lto = true            # Link-time optimization
codegen-units = 1     # Better optimization, slower build

[profile.bench]
inherits = "release"
```

### Conditional Compilation

```rust
// Platform-specific code
#[cfg(target_os = "linux")]
fn platform_specific() {
    // Linux-only code
}

#[cfg(target_os = "macos")]
fn platform_specific() {
    // macOS-only code
}

// Feature flags
#[cfg(feature = "debug")]
fn debug_print(msg: &str) {
    println!("{}", msg);
}

#[cfg(not(feature = "debug"))]
fn debug_print(_msg: &str) {
    // Empty
}
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_identifier() {
        let mut lexer = Lexer::new("hello");
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs

use zen_core::{Lexer, Parser, Compiler};

#[test]
fn test_full_compilation() {
    let source = r#"
        fn main() -> i32 {
            println("Hello!")
            return 0
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap();

    assert!(!program.statements.is_empty());
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_lexer_identifier

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release

# Run specific test file
cargo test --test integration_test
```

### Benchmark Tests

```rust
// benches/lexer_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use zen_core::Lexer;

fn bench_lexer(c: &mut Criterion) {
    let source = include_str!("../../examples/test_full.zen");
    c.bench_function("lexer", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source));
            black_box(lexer.tokenize())
        })
    });
}

criterion_group!(benches, bench_lexer);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench
```

### Test Coverage

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --html

# Generate coverage in terminal
cargo llvm-cov --text
```

---

## Debugging

### Debug Logging

```rust
// Simple debug prints
println!("Debug: {:?}", variable);

// Use dbg! macro for quick debugging
let result = dbg!(add(10, 20));  // Prints value and line number

// Conditional debug logging
#[cfg(debug_assertions)]
println!("Debug info: {:?}", data);
```

### Using lldb/gdb

```bash
# Build debug symbols
cargo build

# Run with debugger
lldb target/debug/zen
(gdb) run compile examples/hello.zen
(gdb) break lexer.rs:42
(gdb) continue
```

### VS Code Debugging

`.vscode/launch.json`:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug zen",
            "cargo": {
                "args": [
                    "build"
                ]
            },
            "args": [
                "compile",
                "examples/hello.zen"
            ]
        }
    ]
}
```

### Debugging Generated LLVM IR

```rust
// In codegen.rs
pub fn generate(&mut self, program: &Program) -> String {
    let ir = self.generate_program(program);

    // Print IR for debugging
    #[cfg(debug_assertions)]
    {
        println!("Generated LLVM IR:");
        println!("{}", ir);
    }

    ir
}
```

---

## Performance Profiling

### CPU Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph -- compile examples/test_full.zen

# Result: flamegraph.svg
```

### Memory Profiling

```bash
# Use valgrind on Linux
valgrind --leak-check=full target/debug/zen compile test.zen

# Use Instruments on macOS
instruments -t "Allocations" target/release/zen compile test.zen
```

### Benchmarking

```rust
use std::time::Instant;

fn benchmark_compilation() {
    let source = include_str!("examples/test_full.zen");

    let start = Instant::now();
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let duration = start.elapsed();

    println!("Lexing took: {:?}", duration);
}
```

---

## Release Process

### Version Bump

1. Update version in `Cargo.toml`
2. Update version in README and docs
3. Create git tag
4. Push tag to trigger release workflow

```bash
# Bump version
vim Cargo.toml  # Update version to 0.1.0

# Commit changes
git add .
git commit -m "chore: bump version to 0.1.0"

# Create tag
git tag -a v0.1.0 -m "Release v0.1.0"

# Push tag
git push origin v0.1.0
```

### Release Checklist

- [ ] All tests passing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Release notes prepared
- [ ] Binary builds for all platforms
- [ ] GitHub release created

### Release Notes Template

```markdown
## v0.1.0

### Added
- Feature 1
- Feature 2

### Changed
- Breaking change 1
- Improvement 1

### Fixed
- Bug fix 1
- Bug fix 2

### Migration Guide
If upgrading from v0.0.1...

### Download
[Assets...]
```

---

## Common Tasks

### Adding a New Token

1. Add to `TokenKind` enum in `src/token/token.rs`
2. Update lexer pattern in `src/lexer/lexer.rs`
3. Add tests in `tests/lexer_tests.rs`

```rust
// src/token/token.rs
pub enum TokenKind {
    // ... existing tokens
    DollarSign,  // New token
}

// src/lexer/lexer.rs
'$' => {
    self.advance();
    self.make_token(TokenKind::DollarSign)
}
```

### Adding a New AST Node

1. Add to appropriate enum in `src/ast/`
2. Update parser to construct the node
3. Update type checker to handle the node
4. Update code generator for the node

```rust
// src/ast/expression.rs
pub enum Expression {
    // ... existing expressions
    DollarExpr(DollarExpr),  // New expression type
}

pub struct DollarExpr {
    pub value: Box<Expression>,
}
```

### Adding a New Language Feature

1. Update lexer (if new tokens needed)
2. Update parser (new grammar rules)
3. Update type checker (type rules)
4. Update code generator (LLVM IR)
5. Add examples
6. Add tests
7. Update documentation

### Debugging Parsing Errors

```rust
// src/parser/parser.rs
fn parse_expression(&mut self) -> Result<Expression, ParseError> {
    // ... parsing logic

    // Debug: print token stream
    #[cfg(debug_assertions)]
    {
        println!("Current token: {:?}", self.current_token());
    }

    // ... rest of parsing
}
```

### Optimizing Compilation Speed

```rust
// Use lazy_static or once_cell for expensive initialization
use once_cell::sync::Lazy;

static KEYWORDS: Lazy<HashMap<&str, TokenKind>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("fn", TokenKind::Fn);
    // ... more keywords
    map
});
```

---

## Resources

### Documentation

- [Rust Book](https://doc.rust-lang.org/book/)
- [The Rust Reference](https://doc.rust-lang.org/reference/)
- [LLVM Language Reference](https://llvm.org/docs/LangRef.html)

### Tools

- [cargo-watch](https://github.com/passcod/cargo-watch) - Watch for changes
- [cargo-expand](https://github.com/dtolnay/cargo-expand) - Macro expansion
- [flamegraph](https://github.com/flamegraph-rs/flamegraph) - Performance profiling

### Community

- [Rust Users Forum](https://users.rust-lang.org/)
- [r/rust on Reddit](https://reddit.com/r/rust)
- [Zen Discord](https://discord.gg/zen-lang)

---

For more information:
- [Contributing Guide](CONTRIBUTING.md)
- [Architecture Guide](ARCHITECTURE.md)
- [Language Reference](LANGUAGE_REFERENCE.md)

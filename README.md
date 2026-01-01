<div align="center">

# Zen Programming Language

**Modern Systems Programming Made Simple**

[![CI](https://github.com/Luvion1/zen-lang/actions/workflows/ci.yml/badge.svg)](https://github.com/Luvion1/zen-lang/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/Luvion1/zen-lang?include_prereleases)](https://github.com/Luvion1/zen-lang/releases)

*Combining the memory safety of Rust, the performance of C++, and the simplicity of Go*

[**Quick Start**](#quick-start) • [**Documentation**](#documentation) • [**Examples**](examples/) • [**Contributing**](#contributing)

</div>

---

## Overview

Zen is a modern systems programming language designed for **performance**, **safety**, and **developer productivity**. It provides compile-time memory safety guarantees without garbage collection, while maintaining a clean and intuitive syntax.

### Key Features

| Feature | Description |
|---------|-------------|
| 🚀 **Zero-Cost Performance** | C/C++ level performance with modern optimizations |
| 🛡️ **Memory Safety** | Compile-time guarantees without runtime overhead |
| 📝 **Clean Syntax** | Intuitive design inspired by Rust and Go |
| 🔥 **Modern Tooling** | Built-in package manager, formatter, and LSP |
| 🎯 **Explicit Control** | Clear ownership semantics and mutability |

### Language Comparison

| Language | Performance | Safety | Simplicity | GC-Free | Learning Curve |
|----------|-------------|--------|------------|---------|----------------|
| C/C++    | ⭐⭐⭐⭐⭐     | ⭐       | ⭐⭐         | ✅      | ⭐⭐             |
| Rust     | ⭐⭐⭐⭐⭐     | ⭐⭐⭐⭐⭐  | ⭐⭐         | ✅      | ⭐⭐             |
| Go       | ⭐⭐⭐⭐      | ⭐⭐⭐⭐    | ⭐⭐⭐⭐       | ❌      | ⭐⭐⭐⭐          |
| **Zen**  | ⭐⭐⭐⭐⭐     | ⭐⭐⭐⭐⭐  | ⭐⭐⭐⭐      | ✅      | ⭐⭐⭐⭐          |

---

## Quick Start

### Installation

**One-line installation:**

```bash
curl -sSL https://raw.githubusercontent.com/Luvion1/zen-lang/main/install.sh | bash
```

**Manual installation:**

```bash
git clone https://github.com/Luvion1/zen-lang.git
cd zen-lang
cargo build --release
sudo cp target/release/zen /usr/local/bin/
```

### Your First Program

Create `hello.zen`:

```zen
fn main() -> i32 {
    println("Hello, Zen!")
    return 0
}
```

Run it:

```bash
zen run hello.zen
# Output: Hello, Zen!
```

---

## Language Features

### Variables & Types

```zen
fn main() -> i32 {
    // Immutable by default
    let name = "Zen"
    let count: i32 = 42
    const PI: f64 = 3.14159

    // Mutable variables
    let mut counter = 0
    counter = counter + 1

    // Type inference
    let x = 10        // inferred as i32
    let pi = 3.14159  // inferred as f64

    return 0
}
```

### Functions

```zen
fn add(a: i32, b: i32) -> i32 {
    a + b  // implicit return
}

fn greet(name: str) -> void {
    println("Hello, {name}!")
}

fn main() -> i32 {
    let result = add(10, 20)
    greet("World")
    return 0
}
```

### Control Flow

```zen
fn main() -> i32 {
    let x = 10

    // If/Else
    if x > 5 {
        println("Big")
    } else {
        println("Small")
    }

    // While Loop
    let mut i = 0
    while i < 5 {
        println("Count: {i}")
        i = i + 1
    }

    // For Loop (C-style)
    for (let mut j = 0; j < 3; j = j + 1) {
        println("Iteration: {j}")
    }

    return 0
}
```

### String Interpolation

```zen
fn main() -> i32 {
    let name = "Alice"
    let age = 25
    let score = calculate_score()
    
    println("Hello, {name}! You are {age} years old.")
    println("Your score is: {score}")
    println("Double score: {multiply(score, 2)}")
    
    return 0
}
```

### Complex Expressions

```zen
fn add(a: i32, b: i32) -> i32 { a + b }
fn multiply(x: i32, y: i32) -> i32 { x * y }

fn main() -> i32 {
    // Nested function calls
    let result = multiply(add(5, 3), add(2, 4))
    
    // Function calls in expressions
    let sum = add(10, 20) + add(5, 15)
    
    println("Result: {result}, Sum: {sum}")
    return 0
}
```

---

## Compiler Architecture

The Zen compiler follows a traditional multi-pass architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                    Zen Compiler Pipeline                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Source Code (.zen) - High-level, readable syntax    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Lexer (Tokenizer) - Converts characters to tokens    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Parser - Builds Abstract Syntax Tree (AST)           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Type Checker - Validates types, performs inference    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Code Generator - Produces LLVM Intermediate           │
│                     Representation (IR)                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  LLVM Compiler (llc) - IR to machine code            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Linker (lld) - Produces native executable           │
└─────────────────────────────────────────────────────────────┘
```

### Component Status

| Component | Status | Tests | Description |
|-----------|--------|--------|-------------|
| **Lexer** | ✅ Complete | 7/7 | Tokenizes keywords, operators, literals |
| **Parser** | ✅ Complete | 16/16 | Parses tokens into AST |
| **Type Checker** | ✅ Complete | - | Static type checking with inference |
| **Code Generator** | ✅ Complete | - | LLVM IR generation (all basic features) |
| **String Interpolation** | ✅ Complete | - | Variable and expression interpolation |
| **For Loops** | ✅ Complete | - | C-style for loops with initialization |
| **Ownership Checker** | ⏳ Planned | - | Memory safety enforcement (v0.1.0+) |
| **Standard Library** | ⏳ Planned | - | I/O, collections, core types (v0.2.0+) |

---

## CLI Usage

```bash
# Compile to native binary
zen compile input.zen -o output

# Compile and run (like `go run`)
zen run input.zen

# Tokenize source code (for debugging)
zen tokenize input.zen

# Display help
zen --help
```

---

## Project Status

### v0.0.1 - Current Release

**Implemented** ✅

- **Core Compiler**
  - Lexer and tokenizer (7/7 tests passing)
  - Parser with full grammar support (16/16 tests passing)
  - Type system with inference
  - Complete code generation (LLVM IR)
  - CLI with `compile`, `run`, `tokenize` commands

- **Language Features**
  - Function definitions with parameters and return types
  - Variable declarations (`let`, `let mut`)
  - Control flow (if/else, while, **for loops**)
  - Expression statements with complex function calls
  - Return statements
  - **String interpolation** (`"Hello, {name}!"`)
  - **Complex function calls** as arguments and in expressions

**New in v0.0.1** 🆕
- ✅ **String Interpolation**: `"The result is {variable}!"` and `"Sum: {add(x, y)}"`
- ✅ **Complex Function Calls**: `multiply(add(5, 3), add(2, 4))`
- ✅ **For Loops**: `for (let mut i = 0; i < 10; i = i + 1) { ... }`
- ✅ **Function Calls in Expressions**: `let sum = add(10, 20) + add(5, 15)`

**Statistics**
- ~3,500 lines of Rust code
- 23/23 tests passing
- Supports: Linux, macOS (Intel x86_64), Windows

### Roadmap

- [**v0.1.0**](docs/ROADMAP.md#v010---complete-core) - Complete Codegen + Ownership
- [**v0.2.0**](docs/ROADMAP.md#v020---standard-library) - Standard Library (Vec, Map, I/O)
- [**v0.3.0**](docs/ROADMAP.md#v030---polish--tooling) - Borrow Checking + Error Messages
- [**v0.5.0**](docs/ROADMAP.md#v050---advanced-features) - Advanced Features (Generics, Traits)
- [**v1.0.0**](docs/ROADMAP.md#v100---feature-complete) - Feature Complete

See [ROADMAP.md](docs/ROADMAP.md) for detailed roadmap.

---

## Development

### Building from Source

#### Prerequisites

- **Rust** 1.70 or higher
- **LLVM** toolchain (llc, lld)
- **GCC** or Clang

#### Build Commands

```bash
# Clone repository
git clone https://github.com/Luvion1/zen-lang.git
cd zen-lang

# Build in release mode (optimized)
cargo build --release

# Run tests
cargo test

# Install globally
cargo install --path .
```

#### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_lexer_identifier
```

---

## Contributing

We welcome contributions from everyone! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

### How to Contribute

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes** with tests and documentation
4. **Test your changes**: `cargo test && cargo fmt && cargo clippy`
5. **Commit your changes**: `git commit -m "feat: add amazing feature"`
6. **Push and create Pull Request**

### Areas for Contribution

#### High Priority

- 🎯 **Ownership System**
  - Ownership tracking engine
  - Move semantics with `<-` operator
  - Simplified borrow checking
  - Zero-cost abstractions

#### Medium Priority

- 📚 **Standard Library**
  - Vec<T> implementation
  - I/O functions
  - String operations

- 🎨 **Error Messages**
  - Better diagnostics
  - Suggested fixes
  - Color-coded output

#### Low Priority

- 📖 **Documentation**
  - Tutorials
  - API documentation
  - Examples

- ⚡ **Performance**
  - Compilation speed
  - Binary size optimization
  - Runtime performance

---

## Documentation

- 📖 [Language Reference](docs/LANGUAGE_REFERENCE.md) - Complete language specification
- 🏗️ [Architecture Guide](docs/ARCHITECTURE.md) - Compiler architecture deep dive
- 👥 [Contributing Guide](docs/CONTRIBUTING.md) - How to contribute
- 🗺️ [Roadmap](docs/ROADMAP.md) - Future plans and milestones
- 💡 [Examples](examples/) - Sample programs

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Community

- 💬 **Discussions**: [GitHub Discussions](https://github.com/Luvion1/zen-lang/discussions)
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/Luvion1/zen-lang/issues)
- 📖 **Documentation**: [docs/](docs/)
- ⭐ **Star us**: [GitHub](https://github.com/Luvion1/zen-lang)

---

<div align="center">

**Built with ❤️ by the Zen Community**

[⬆ Back to Top](#zen-programming-language)

</div>

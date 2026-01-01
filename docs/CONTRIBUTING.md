# Contributing to Zen

Thank you for your interest in contributing to Zen! We welcome contributions from everyone.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

- Be respectful and inclusive
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Git
- LLVM toolchain (llc, lld, clang)

### Setup Development Environment

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/zen-lang.git
cd zen-lang

# Add upstream remote
git remote add upstream https://github.com/Lunar-Chipter/zen-lang.git

# Install development tools
rustup component add rustfmt clippy rust-analyzer

# Build in development mode
cargo build

# Run tests
cargo test
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-fix-name
```

### 2. Make Your Changes

- Write code following [Coding Standards](#coding-standards)
- Add tests for new functionality
- Update documentation
- Ensure all existing tests pass

### 3. Test Your Changes

```bash
# Run tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run all checks
cargo check
```

### 4. Commit Your Changes

```bash
git add .
git commit -m "feat: add new feature description"
```

**Commit Message Format:**

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements

**Examples:**

```
feat(parser): add support for for loops

Implements parsing of C-style for loops with
counter variable initialization.

Closes #123
```

```
fix(codegen): correct LLVM IR generation for arrays

Fixes issue where array initialization generated
incorrect IR, causing runtime errors.

Fixes #456
```

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub with:
- Clear title and description
- Reference related issues
- Screenshots if applicable
- Testing instructions

## Coding Standards

### Rust Code Style

- Use `cargo fmt` for formatting
- Follow Rust API guidelines
- Use meaningful variable and function names
- Add documentation comments (`///`) for public APIs
- Keep functions focused and small (<50 lines when possible)

### Documentation

```rust
/// Tokenizes the source code into a vector of tokens.
///
/// # Arguments
///
/// * `source` - A string slice containing the source code
///
/// # Returns
///
/// Returns a vector of `Token` representing the lexed source
///
/// # Examples
///
/// ```
/// let lexer = Lexer::new("fn main() {}");
/// let tokens = lexer.tokenize();
/// ```
pub fn tokenize(&mut self) -> Vec<Token> {
    // implementation
}
```

### Error Handling

- Use `anyhow::Result` for application errors
- Use `thiserror` for library errors
- Provide helpful error messages
- Include context where appropriate

```rust
use anyhow::{Context, Result};

pub fn compile_file(path: &str) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .context("Failed to read source file")?;

    let lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    // ... rest of compilation
    Ok(())
}
```

## Testing

### Unit Tests

Write unit tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_hello_world() {
        let source = "fn main() {}";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].kind, TokenKind::Fn);
    }
}
```

### Integration Tests

Create integration tests in `tests/` directory:

```rust
// tests/integration_test.rs

use zen_core::{Lexer, Parser};

#[test]
fn test_full_compilation() {
    let source = r#"
        fn main() -> i32 {
            println("Hello, World!")
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

### Test Organization

```
tests/
├── integration/
│   ├── lexer_tests.rs
│   ├── parser_tests.rs
│   └── codegen_tests.rs
└── examples/
    └── test_examples.rs
```

## Areas for Contribution

### High Priority

1. **Code Generation Fixes**
   - Complex function calls
   - For loop variable initialization
   - String formatting

2. **Ownership System**
   - Ownership tracking
   - Move semantics
   - Borrow checking (simplified)

3. **Standard Library**
   - Vec<T> implementation
   - I/O functions
   - String operations

### Medium Priority

4. **Error Messages**
   - Better diagnostics
   - Suggested fixes
   - Color-coded output

5. **Documentation**
   - Language reference
   - Tutorials
   - API documentation

6. **Testing**
   - Increase test coverage
   - Add integration tests
   - Benchmark suite

### Low Priority

7. **Tooling**
   - Linter
   - Formatter
   - IDE integration

8. **Performance**
   - Compilation speed
   - Binary size optimization
   - Runtime performance

## Submitting Changes

### Pull Request Checklist

- [ ] Code follows project style guide
- [ ] Tests pass locally (`cargo test`)
- [ ] Linting passes (`cargo clippy`)
- [ ] Formatting applied (`cargo fmt`)
- [ ] Documentation updated
- [ ] Commit messages follow conventional commits
- [ ] PR description is clear and complete

### Review Process

1. Maintainers will review your PR
2. Feedback will be provided if changes are needed
3. PRs may need multiple rounds of review
4. Once approved, PR will be merged

## Reporting Issues

### Bug Reports

Include:
- Description of the bug
- Steps to reproduce
- Expected vs actual behavior
- Environment (OS, Rust version)
- Minimal example code if possible
- Error messages and logs

### Feature Requests

Include:
- Clear description of the feature
- Use case/motivation
- Proposed API (if applicable)
- Examples of how it would work

### Questions

- Use GitHub Discussions for questions
- Search existing discussions first
- Be clear and specific

## Getting Help

- GitHub Discussions: https://github.com/Lunar-Chipter/zen-lang/discussions
- GitHub Issues: https://github.com/Lunar-Chipter/zen-lang/issues
- Documentation: https://github.com/Lunar-Chipter/zen-lang/tree/main/docs

## Recognition

Contributors will be recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project website

Thank you for contributing! 🎉

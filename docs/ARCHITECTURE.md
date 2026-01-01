# Zen Compiler Architecture

This document provides a deep dive into the Zen compiler architecture, design decisions, and implementation details.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Components](#components)
- [Data Flow](#data-flow)
- [Design Decisions](#design-decisions)
- [Performance Considerations](#performance-considerations)
- [Future Improvements](#future-improvements)

## Overview

The Zen compiler is written in Rust and compiles Zen source code to native binaries using LLVM as the backend. The compiler follows a traditional multi-pass architecture with clear separation of concerns.

**Goals:**
- Zero-cost abstractions
- Compile-time safety guarantees
- Clean, maintainable code
- Fast compilation times
- Helpful error messages

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Zen Compiler Pipeline                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Source Code (.zen)                                          │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ fn main() -> i32 {                                   │    │
│  │     println("Hello, Zen!")                           │    │
│  │     return 0                                         │    │
│  │ }                                                     │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Lexer (Tokenizer)                                          │
│  - Character by character scanning                          │
│  - Token recognition                                         │
│  - Line/column tracking                                      │
│  Output: Vec<Token>                                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Parser                                                     │
│  - Recursive descent parsing                                 │
│  - Operator precedence handling                              │
│  - AST construction                                          │
│  Output: Program (AST)                                      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Type Checker                                               │
│  - Type inference                                            │
│  - Type validation                                           │
│  - Type unification                                           │
│  Output: Typed AST (with type annotations)                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Ownership Checker (Planned)                                 │
│  - Ownership tracking                                        │
│  - Borrow checking                                           │
│  - Lifetime inference                                        │
│  Output: Ownership-validated AST                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Code Generator                                              │
│  - Manual LLVM IR generation                                 │
│  - String-based IR construction                              │
│  - No external LLVM dependencies (inkwell)                 │
│  Output: LLVM IR (.ll)                                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  LLVM Compiler (llc)                                         │
│  - IR to machine code                                        │
│  - Optimization passes                                       │
│  Output: Object File (.o)                                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Linker (lld)                                                │
│  - Object file linking                                       │
│  - Standard library linking                                  │
│  Output: Native Binary                                       │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. Lexer (`src/lexer/`)

**Purpose:** Convert source code into a stream of tokens

**Key Data Structures:**
```rust
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

pub enum TokenKind {
    // Keywords
    Fn, Let, Mut, Return, If, Else, While, For, Match,

    // Types
    Int32, Int64, Float32, Float64, Bool, Str, Char, Void,

    // Literals
    Identifier, StringLiteral, IntegerLiteral, FloatLiteral,

    // Operators
    Plus, Minus, Asterisk, Slash, Equal, LessThan, GreaterThan,

    // Delimiters
    LeftParen, RightParen, LeftBrace, RightBrace,
    LeftBracket, RightBracket, Comma, Colon, Semicolon,

    // Other
    ArrowRight, EOF,
}
```

**Algorithm:**
1. Read characters from source
2. Match against patterns (keywords, operators, literals)
3. Create token with position information
4. Continue until EOF

**Design Choices:**
- Single-pass lexer (no backtracking)
- Minimal state machine
- Position tracking for error reporting

### 2. Parser (`src/parser/`)

**Purpose:** Parse tokens into Abstract Syntax Tree (AST)

**Key Data Structures:**
```rust
pub struct Program {
    pub statements: Vec<Statement>,
}

pub enum Statement {
    FunctionDecl(FunctionDecl),
    VariableDecl(VariableDecl),
    IfStatement(IfStatement),
    WhileStatement(WhileStatement),
    ReturnStatement(ReturnStatement),
    Expression(Expression),
}

pub enum Expression {
    BinaryExpr(BinaryExpr),
    UnaryExpr(UnaryExpr),
    Literal(Literal),
    Identifier(Identifier),
    FunctionCall(FunctionCall),
}
```

**Algorithm:**
- Recursive descent parsing
- Precedence climbing for expressions
- Error recovery with panic mode

**Design Choices:**
- Clear separation between statements and expressions
- Minimal lookahead (1 token)
- Type information attached to AST nodes

### 3. Type Checker (`src/typechecker/`)

**Purpose:** Validate type correctness and infer types

**Key Data Structures:**
```rust
pub enum Type {
    Void,
    Int(IntSize),
    Float(FloatSize),
    Bool,
    Str,
    Char,
    Struct(String),
}

pub struct TypeEnvironment {
    scopes: Vec<HashMap<String, Type>>,
}

pub struct TypeChecker {
    env: TypeEnvironment,
}
```

**Algorithm:**
1. Traverse AST in multiple passes
2. Collect function signatures
3. Check statement types
4. Infer types where not explicit
5. Validate type compatibility

**Design Choices:**
- Hindley-Milner type inference
- Type unification algorithm
- No type erasure (types kept for codegen)

### 4. Code Generator (`src/codegen/`)

**Purpose:** Generate LLVM IR from typed AST

**Key Data Structures:**
```rust
pub struct CodeGenerator {
    functions: Vec<String>,
    current_function: String,
    counter: usize,
}

pub struct IRBuilder {
    buffer: String,
    indent: usize,
}
```

**Algorithm:**
1. Generate LLVM IR as strings
2. Use IRBuilder helper for indentation/formatting
3. Generate function declarations first
4. Generate function bodies
5. Emit to temporary .ll file

**Design Choices:**
- String-based IR generation (no inkwell)
- Simpler dependency chain
- Manual optimization hints
- Explicit stack allocation

**Why Manual IR Generation?**
- No LLVM FFI complexity
- Faster compilation
- More control over IR structure
- Easier debugging of generated code

### 5. Compiler Driver (`src/compiler.rs`)

**Purpose:** Orchestrate the compilation pipeline

**Algorithm:**
1. Read source file
2. Run lexer → get tokens
3. Run parser → get AST
4. Run type checker → validate types
5. Run code generator → get LLVM IR
6. Call llc → get object file
7. Call linker → get binary
8. Cleanup temporary files

**Design Choices:**
- Clean separation of concerns
- Each stage can be tested independently
- Error propagation with context
- Temporary files in system temp directory

## Data Flow

### Example: Hello World

**Source:**
```zen
fn main() -> i32 {
    println("Hello, Zen!")
    return 0
}
```

**Lexer Output:**
```
Token{Fn, "fn", 1, 1}
Token{Identifier, "main", 1, 4}
Token{LeftParen, "(", 1, 8}
Token{RightParen, ")", 1, 9}
Token{ArrowRight, "->", 1, 11}
Token{Int32, "i32", 1, 14}
Token{LeftBrace, "{", 1, 18}
...
```

**Parser Output (AST):**
```
Program {
    statements: [
        FunctionDecl {
            name: "main",
            params: [],
            return_type: Int32,
            body: [
                Expression(
                    FunctionCall {
                        name: "println",
                        args: [StringLiteral("Hello, Zen!")]
                    }
                ),
                ReturnStatement(
                    Literal(0)
                )
            ]
        }
    ]
}
```

**Type Checker Output:**
- All types validated
- No type errors

**Code Generator Output (LLVM IR):**
```llvm
define i32 @main() {
entry:
  %0 = call i32 @println(i8* getelementptr([13 x i8], [13 x i8]* @.str, i32 0, i32 0))
  ret i32 0
}

declare i32 @println(i8*)
```

**Final Binary:**
```
Hello, Zen!
```

## Design Decisions

### 1. No Lifetime Annotations

**Decision:** Zen will not require explicit lifetime annotations like Rust.

**Rationale:**
- Reduces learning curve
- Simplified ownership model
- Auto-inference for most cases
- Fallback to explicit `<-` for transfers

**Trade-off:**
- Less fine-grained control
- May require more runtime checks in edge cases

### 2. Manual LLVM IR Generation

**Decision:** Generate LLVM IR as strings instead of using LLVM FFI (inkwell).

**Rationale:**
- Simpler dependency chain
- Faster compilation
- More control over IR structure
- Easier to debug generated code

**Trade-off:**
- No type-safe IR generation
- More error-prone
- Manual maintenance of IR format

### 3. String-Based Error Messages

**Decision:** Implement custom error message system instead of using crates like miette.

**Rationale:**
- Full control over message format
- No external dependencies
- Tailored to Zen's needs

**Trade-off:**
- More boilerplate
- Must implement features manually

### 4. Single Pass Type Checking

**Decision:** Type checking done in single pass for simplicity.

**Rationale:**
- Simpler implementation
- Faster type checking
- Clearer error messages

**Trade-off:**
- No mutually recursive types
- Limited type inference

## Performance Considerations

### Compilation Speed

**Current:**
- ~1s for simple programs
- ~5s for medium programs (1000 lines)
- ~30s for large programs (10,000+ lines)

**Optimization Targets:**
- <500ms for simple programs
- <2s for medium programs
- <10s for large programs

**Optimization Strategies:**
1. Parallelize independent compilation units
2. Cache lexer/parser results
3. Incremental compilation
4. Faster IR generation

### Binary Size

**Current:**
- ~10KB for simple programs
- ~50KB for medium programs

**Optimization Targets:**
- <5KB for simple programs
- <25KB for medium programs

**Optimization Strategies:**
1. LTO (Link-Time Optimization)
2. Dead code elimination
3. Smaller standard library
4. Strip debug symbols

### Runtime Performance

**Current:** Not yet benchmarked

**Target:** Within 5% of C++ on standard benchmarks

## Future Improvements

### Short-term (0.1.0 - 0.2.0)

1. **Complete Code Generation**
   - All operators
   - All control flow
   - Complex function calls

2. **Ownership System**
   - Move semantics
   - Borrow checking
   - Lifetimes (simplified)

3. **Standard Library**
   - Vec<T>
   - I/O functions
   - String operations

### Medium-term (0.3.0 - 0.5.0)

1. **Advanced Features**
   - Generics
   - Traits
   - Pattern matching

2. **Tooling**
   - Package manager
   - Linter
   - Formatter

3. **Ecosystem**
   - Documentation generator
   - Testing framework
   - Build system

### Long-term (1.0.0+)

1. **Compiler Optimizations**
   - Incremental compilation
   - Parallel compilation
   - Better error recovery

2. **Language Features**
   - Async/await
   - Macros
   - Modules

3. **Platform Support**
   - WebAssembly
   - Embedded systems
   - Android/iOS

## Conclusion

The Zen compiler is designed to be simple, fast, and maintainable. By following clear separation of concerns and making deliberate design trade-offs, we aim to create a compiler that is both powerful and approachable.

For more information:
- [Language Reference](LANGUAGE_REFERENCE.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Roadmap](ROADMAP.md)

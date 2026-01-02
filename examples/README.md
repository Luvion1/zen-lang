# Zen Programming Language Examples

This directory contains comprehensive examples showcasing Zen's features and capabilities.

## Basic Examples

### 1. **hello.zen** - Hello World
The classic first program in any language.
```bash
zen run examples/hello.zen
```

### 2. **variables.zen** - Variables and Type Inference
Demonstrates variable declarations, type inference, and mutability.
- Type inference from literals
- Explicit type annotations
- Mutable variables
- All primitive types (i32, f64, bool, str, char)

### 3. **arithmetic.zen** - Basic Arithmetic
Simple mathematical operations and expressions.

## Intermediate Examples

### 4. **functions_advanced.zen** - Advanced Functions
Comprehensive function examples including:
- Function parameters and return types
- Nested function calls
- Recursive functions (factorial)
- Void functions
- Complex expressions with function calls

### 5. **control_flow_advanced.zen** - Control Flow
All control flow structures:
- If-else statements
- While loops
- C-style for loops with initialization
- Match expressions with multiple patterns

### 6. **math_operations.zen** - Mathematical Operations
Complete mathematical operations showcase:
- Integer and float arithmetic
- Comparison operations
- Boolean logic operations
- Mixed-type expressions

### 7. **strings_chars.zen** - String and Character Handling
String and character operations:
- String variables and assignment
- Character literals
- String type inference
- Multiple string manipulations

### 8. **string_interpolation.zen** - String Interpolation
Modern string formatting with variable embedding:
- Variable interpolation: `"Hello, {name}!"`
- Expression interpolation: `"Sum: {add(x, y)}"`
- Mixed text and variables
- Function calls in strings

## Advanced Examples

### 9. **complex_expressions.zen** - Complex Expressions
Advanced expression handling:
- Nested arithmetic expressions
- Complex function call chains
- Boolean expression combinations
- Function calls in conditions

### 11. **algorithms.zen** - Algorithms and Data Processing
Real-world algorithms implementation:
- Fibonacci sequence (recursive)
- Greatest Common Divisor (Euclidean algorithm)
- Prime number checking
- Range summation
- Factorial with loops

### 12. **pattern_matching.zen** - Pattern Matching
Advanced pattern matching and conditional logic:
- Match expressions with multiple cases
- Conditional grading system
- Day-of-week matching
- Complex boolean conditions

## Application Examples

### 13. **guessing_game.zen** - Number Guessing Game
Interactive game simulation demonstrating:
- Game logic implementation
- Distance calculation
- Feedback system
- Multiple function interactions

### 14. **calculator.zen** - Scientific Calculator
Full-featured calculator with:
- Basic arithmetic operations (+, -, *, /)
- Power calculations
- Error handling (division by zero)
- Complex mathematical expressions
- Float number operations

## Running Examples

```bash
# Run any example
zen run examples/<filename>.zen

# Or compile first, then run
zen compile examples/<filename>.zen
./<filename>
```

## Features Demonstrated

- ✅ **Type System**: Inference, explicit types, primitives
- ✅ **Functions**: Parameters, returns, recursion, nesting
- ✅ **Control Flow**: If/else, loops, match expressions
- ✅ **Variables**: Immutable/mutable, all types
- ✅ **Expressions**: Complex nested expressions
- ✅ **Algorithms**: Real-world problem solving
- ✅ **Applications**: Complete programs

## Performance Notes

All examples compile to native machine code via LLVM, providing:
- Zero-cost abstractions
- Optimal performance
- Small binary sizes
- Fast execution times

## Next Steps

These examples showcase Zen v0.0.1 capabilities. Future versions will add:
- Structs and methods
- Arrays and collections
- Error handling
- Standard library functions
- Ownership system improvements

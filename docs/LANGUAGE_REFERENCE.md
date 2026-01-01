# Zen Language Reference

Complete language specification for Zen programming language.

## Table of Contents

- [Overview](#overview)
- [Lexical Structure](#lexical-structure)
- [Types](#types)
- [Variables](#variables)
- [Functions](#functions)
- [Control Flow](#control-flow)
- [Structs](#structs)
- [Ownership](#ownership)
- [Expressions](#expressions)
- [Statements](#statements)
- [Standard Library](#standard-library)
- [Grammar](#grammar)

## Overview

Zen is a statically-typed, compiled systems programming language with:

- Compile-time memory safety
- Zero-cost abstractions
- No garbage collector
- Clean, modern syntax

```zen
fn main() -> i32 {
    let greeting = "Hello, Zen!"
    println(greeting)
    return 0
}
```

---

## Lexical Structure

### Comments

```zen
// Single-line comment

/*
   Multi-line comment
   Spans multiple lines
*/
```

### Whitespace

Zen is whitespace-insensitive. Whitespace is used for separation, not significance.

### Identifiers

```zen
// Valid identifiers
let name = "Zen"
let _internal = 42
let camelCase = true

// Invalid identifiers (keywords)
// let fn = 1  // Error: reserved keyword
```

### Keywords

**Reserved Keywords:**
```
fn, let, mut, const, if, else, while, for, match,
return, struct, enum, impl, trait, use, mod, crate,
pub, true, false, void, box, self, super
```

### Literals

```zen
// Integer literals
let decimal = 42
let hex = 0xFF
let octal = 0o755
let binary = 0b1010

// Floating-point literals
let float1 = 3.14
let float2 = 6.022e23

// String literals
let string = "Hello"
let escaped = "Line 1\nLine 2"

// Character literals
let char = 'Z'
let unicode = '🦀'

// Boolean literals
let truth = true
let falsity = false
```

---

## Types

### Primitive Types

#### Integer Types

```zen
// Signed integers
let a: i8 = 127           // 8-bit signed
let b: i16 = 1000         // 16-bit signed
let c: i32 = 100000       // 32-bit signed (default)
let d: i64 = 1000000000   // 64-bit signed

// Unsigned integers
let e: u8 = 255           // 8-bit unsigned
let f: u16 = 1000         // 16-bit unsigned
let g: u32 = 100000       // 32-bit unsigned
let h: u64 = 1000000000   // 64-bit unsigned
```

#### Floating-Point Types

```zen
let a: f32 = 3.14         // 32-bit floating-point
let b: f64 = 3.14159      // 64-bit floating-point (default)
```

#### Other Primitive Types

```zen
// Boolean
let flag: bool = true

// String (UTF-8)
let text: str = "Hello, Zen!"

// Character (Unicode scalar value)
let ch: char = 'Z'

// Void (no value)
fn no_return() -> void {
    println("No return value")
}
```

### Type Inference

```zen
// Type is inferred from expression
let count = 42        // Inferred as i32
let pi = 3.14159     // Inferred as f64
let name = "Zen"     // Inferred as str

// Explicit type annotation
let count: i32 = 42
```

### Array Types

```zen
// Fixed-size array
let numbers: [i32; 5] = [1, 2, 3, 4, 5]

// Array with same value repeated
let zeros: [i32; 10] = [0; 10]

// Array access
let first = numbers[0]    // 1
```

---

## Variables

### Declaration

```zen
// Immutable (default)
let name = "Zen"
let pi: f64 = 3.14159
const PI: f64 = 3.14159

// Mutable
let mut counter = 0
let mut data: i32 = 42
```

### Assignment

```zen
let mut x = 10

// Simple assignment
x = 20

// Compound assignment (planned)
x += 5    // x = x + 5
x -= 3    // x = x - 3
```

### Shadowing

```zen
let x = 5
let x = x + 1    // New variable, shadows previous

{
    let x = "hello"   // Different type in inner scope
}
```

---

## Functions

### Declaration

```zen
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn greet(name: str) -> void {
    println("Hello, {name}!")
}
```

### Parameters

```zen
// Multiple parameters
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// No parameters
fn greet() -> void {
    println("Hello, World!")
}

// Default parameters (planned)
fn greet(name: str = "World") -> void {
    println("Hello, {name}!")
}
```

### Return Values

```zen
// Implicit return (last expression)
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Explicit return
fn add(a: i32, b: i32) -> i32 {
    return a + b
}

// No return value
fn print_hello() -> void {
    println("Hello, World!")
}
```

### Function Calls

```zen
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() -> i32 {
    // Direct call
    let result = add(10, 20)

    // Nested calls
    let result = add(add(1, 2), add(3, 4))

    // Call as expression
    println(add(10, 20))

    return 0
}
```

### Closures (Planned)

```zen
// Planned for v0.5.0
let add = |a: i32, b: i32| -> i32 {
    a + b
}

let result = add(10, 20)
```

---

## Control Flow

### If/Else

```zen
let x = 10

if x > 5 {
    println("Big")
} else if x > 0 {
    println("Small positive")
} else {
    println("Non-positive")
}

// Expression form
let is_positive = if x > 0 { true } else { false }
```

### While Loop

```zen
let mut i = 0
while i < 10 {
    println(i)
    i = i + 1
}

// Infinite loop
while true {
    // Do something forever
    break
}
```

### For Loop

```zen
// C-style for loop
for (i = 0; i < 10; i = i + 1) {
    println(i)
}

// For loop with mutable counter
for (mut i = 0; i < 10; i = i + 1) {
    i = i * 2
}

// Range-based for loop (planned)
for i in 0..10 {
    println(i)
}

// For loop without body
for (i = 0; i < 10; i = i + 1) { }
```

### Match Expression

```zen
let x = 2

match x {
    1 => println("One"),
    2 => println("Two"),
    3 => println("Three"),
    _ => println("Other")
}

// Match as expression
let description = match x {
    1 => "One",
    2 => "Two",
    _ => "Other"
}

// Match with guards (planned)
match x {
    n if n < 0 => println("Negative"),
    n if n > 0 => println("Positive"),
    _ => println("Zero")
}
```

### Break and Continue (Planned)

```zen
// Planned for v0.2.0
while true {
    break       // Exit loop
    continue    // Skip to next iteration
}

// Nested loops
for i in 0..10 {
    for j in 0..10 {
        if i == j {
            continue
        }
        if i * j > 50 {
            break
        }
    }
}
```

---

## Structs

### Declaration

```zen
struct Point {
    x: i32
    y: i32
}

struct Person {
    name: str
    age: i32
    email: str
}
```

### Instantiation

```zen
let p = Point { x: 10, y: 20 }

let person = Person {
    name: "Alice",
    age: 25,
    email: "alice@example.com"
}
```

### Field Access

```zen
let mut p = Point { x: 10, y: 20 }

let x = p.x      // Read field
p.x = 15         // Error: immutable

let mut p2 = Point { x: 10, y: 20 }
p2.x = 15        // OK: mutable
```

### Tuple Structs (Planned)

```zen
struct Color(i32, i32, i32)

let red = Color(255, 0, 0)
```

### Unit Structs (Planned)

```zen
struct Empty

let e = Empty
```

---

## Ownership

### Core Rules

1. **Every value has one owner**
2. **Ownership transfer is explicit with `<-`**
3. **Small types auto-copy**

### Move Semantics

```zen
struct BigData {
    data: [i32; 1000]
}

fn consume(<-data: BigData) {
    // data is consumed here
}

fn main() -> i32 {
    let data = BigData { data: [0; 1000] }
    consume(<-data)  // Transfer ownership

    // Error: data was moved
    // use(data)

    return 0
}
```

### Copy Semantics

```zen
// Primitives are copy
let a: i32 = 10
let b = a  // Copy, a is still valid
let c = a  // Still OK

// Small structs auto-copy
struct Point {
    x: i32
    y: i32
}

let p1 = Point { x: 10, y: 20 }
let p2 = p1  // Auto copy
// p1 is still valid
```

### Explicit Transfer

```zen
fn transfer(<-value: BigStruct) {
    use(value)
}

fn main() -> i32 {
    let big = BigStruct { ... }
    transfer(<-big)

    // big is no longer accessible
    return 0
}
```

---

## Expressions

### Arithmetic Operators

```zen
// Addition
let sum = 10 + 20

// Subtraction
let diff = 20 - 10

// Multiplication
let product = 10 * 5

// Division
let quotient = 20 / 4

// Modulo (planned)
let remainder = 17 % 5

// Unary minus
let neg = -5
```

### Comparison Operators

```zen
// Equality
let eq = 10 == 10      // true
let ne = 10 != 5       // true

// Less than
let lt = 5 < 10        // true

// Greater than
let gt = 10 > 5        // true

// Less than or equal
let le = 10 <= 10      // true

// Greater than or equal
let ge = 10 >= 5       // true
```

### Logical Operators

```zen
// Logical AND
let and = true && false

// Logical OR
let or = true || false

// Logical NOT
let not = !true
```

### String Interpolation (Planned)

```zen
let name = "Zen"
let count = 42

println("Hello, {name}!")
println("Count: {count}")
println("2 + 2 = {2 + 2}")
```

---

## Statements

### Expression Statements

```zen
fn main() -> i32 {
    // Expression statement
    println("Hello, Zen!")

    // Value discarded
    42
}
```

### Return Statement

```zen
fn main() -> i32 {
    return 0
}

fn add(a: i32, b: i32) -> i32 {
    return a + b
}
```

### Variable Declaration Statement

```zen
fn main() -> i32 {
    let x = 10
    let mut y = 20
    return 0
}
```

---

## Standard Library

### I/O Functions

```zen
// Print with newline
println("Hello, Zen!")

// Print without newline (planned)
print("No newline")

// Formatted print (planned)
printf("Value: %d\n", 42)
```

### File I/O (Planned)

```zen
// Read file
let content = File::read("data.txt")

// Write file
File::write("output.txt", "Hello!")

// Check if file exists
if File::exists("data.txt") {
    println("File exists")
}
```

### Collections (Planned)

```zen
// Vector
let vec = Vec::new()
vec.push(10)
vec.push(20)
let first = vec.get(0)

// Map
let map = Map::new()
map.insert("key", "value")
let value = map.get("key")
```

---

## Grammar

### EBNF Grammar

```
program        = statement*

statement      = function_decl
               | variable_decl
               | if_statement
               | while_statement
               | for_statement
               | match_statement
               | return_statement
               | expression_statement

function_decl  = "fn" identifier "(" [parameter_list] ")" "->" type block

parameter_list = parameter ("," parameter)*
parameter      = identifier ":" type

variable_decl  = "let" ["mut"] identifier [":" type] "=" expression

if_statement   = "if" expression block ["else" if_statement | block]

while_statement = "while" expression block

for_statement  = "for" "(" ["mut"] identifier "=" expression ";"
                expression ";" identifier "=" expression ")" block

match_statement = "match" expression "{" match_arm* "}"
match_arm      = expression "=>" expression [","]

return_statement = "return" [expression] ";"

expression     = assignment

assignment     = identifier "=" assignment | equality

equality       = comparison (("==" | "!=") comparison)*

comparison     = term (("<" | ">" | "<=" | ">=") term)*

term           = factor (("+" | "-") factor)*

factor         = unary (("*" | "/" | "%") unary)*

unary          = ("-" | "!") unary | call

call           = primary "(" [argument_list] ")"

argument_list  = expression ("," expression)*

primary        = integer_literal
               | float_literal
               | string_literal
               | char_literal
               | boolean_literal
               | identifier
               | "(" expression ")"

block          = "{" statement* "}"

type           = identifier
               | "[" type ";" integer_literal "]"
               | "(" [type_list] ")"

type_list      = type ("," type)*

identifier     = letter {letter | digit | "_"}

integer_literal = digit+

float_literal   = digit+ "." digit+

string_literal = '"' {character} '"'

char_literal   = "'" character "'"

boolean_literal = "true" | "false"
```

---

## Version History

| Version | Features |
|---------|----------|
| v0.0.1 | Lexer, Parser, Type Checker, Basic Codegen |
| v0.1.0 | Complete codegen, Ownership system |
| v0.2.0 | Standard library (Vec, Map, I/O) |
| v0.3.0 | Borrow checking, Error messages |
| v0.5.0 | Generics, Traits, Modules |
| v1.0.0 | Feature complete, Stable ABI |

---

For more information:
- [Architecture Guide](ARCHITECTURE.md)
- [Roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)

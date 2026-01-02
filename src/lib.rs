pub mod ast;
pub mod cli;
pub mod codegen;
pub mod compiler;
pub mod lexer;
pub mod ownership;
pub mod parser;
pub mod token;
pub mod typechecker;

use std::fmt;

#[derive(Debug, Clone)]
pub enum ZenError {
    LexError {
        message: String,
        line: usize,
        column: usize,
        source_line: Option<String>,
    },
    ParseError {
        message: String,
        line: usize,
        column: usize,
        source_line: Option<String>,
        expected: Option<String>,
        found: Option<String>,
    },
    TypeError {
        message: String,
        line: usize,
        column: usize,
        source_line: Option<String>,
        expected_type: Option<String>,
        found_type: Option<String>,
    },
    CodegenError {
        message: String,
        context: Option<String>,
    },
    IoError {
        message: String,
        path: Option<String>,
    },
}

impl ZenError {
    pub fn with_source_line(mut self, source_line: String) -> Self {
        match &mut self {
            ZenError::LexError { source_line: ref mut sl, .. } => *sl = Some(source_line),
            ZenError::ParseError { source_line: ref mut sl, .. } => *sl = Some(source_line),
            ZenError::TypeError { source_line: ref mut sl, .. } => *sl = Some(source_line),
            _ => {}
        }
        self
    }

    pub fn format_with_context(&self) -> String {
        match self {
            ZenError::LexError { message, line, column, source_line } => {
                let mut result = format!("Lexical error at {}:{}: {}", line, column, message);
                if let Some(src) = source_line {
                    result.push_str(&format!("\n  {}\n  {}^", src, " ".repeat(column.saturating_sub(1))));
                }
                result
            }
            ZenError::ParseError { message, line, column, source_line, expected, found } => {
                let mut result = format!("Parse error at {}:{}: {}", line, column, message);
                if let (Some(exp), Some(fnd)) = (expected, found) {
                    result.push_str(&format!("\n  Expected: {}\n  Found: {}", exp, fnd));
                }
                if let Some(src) = source_line {
                    result.push_str(&format!("\n  {}\n  {}^", src, " ".repeat(column.saturating_sub(1))));
                }
                result
            }
            ZenError::TypeError { message, line, column, source_line, expected_type, found_type } => {
                let mut result = format!("Type error at {}:{}: {}", line, column, message);
                if let (Some(exp), Some(fnd)) = (expected_type, found_type) {
                    result.push_str(&format!("\n  Expected type: {}\n  Found type: {}", exp, fnd));
                }
                if let Some(src) = source_line {
                    result.push_str(&format!("\n  {}\n  {}^", src, " ".repeat(column.saturating_sub(1))));
                }
                result
            }
            ZenError::CodegenError { message, context } => {
                let mut result = format!("Code generation error: {}", message);
                if let Some(ctx) = context {
                    result.push_str(&format!("\n  Context: {}", ctx));
                }
                result
            }
            ZenError::IoError { message, path } => {
                let mut result = format!("I/O error: {}", message);
                if let Some(p) = path {
                    result.push_str(&format!("\n  Path: {}", p));
                }
                result
            }
        }
    }
}

impl fmt::Display for ZenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZenError::LexError { message, line, column, .. } => write!(f, "Lexical error at {}:{}: {}", line, column, message),
            ZenError::ParseError { message, line, column, .. } => write!(f, "Parse error at {}:{}: {}", line, column, message),
            ZenError::TypeError { message, line, column, .. } => write!(f, "Type error at {}:{}: {}", line, column, message),
            ZenError::CodegenError { message, .. } => write!(f, "Code generation error: {}", message),
            ZenError::IoError { message, .. } => write!(f, "I/O error: {}", message),
        }
    }
}

impl std::error::Error for ZenError {}
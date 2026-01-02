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

#[derive(Debug)]
pub enum ZenError {
    LexError {
        message: String,
        line: usize,
        column: usize,
    },
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },
    TypeError {
        message: String,
        line: usize,
        column: usize,
    },
    CodegenError {
        message: String,
    },
    IoError {
        message: String,
    },
}

impl fmt::Display for ZenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZenError::LexError {
                message,
                line,
                column,
            } => write!(f, "Lexical error at {}:{}: {}", line, column, message),
            ZenError::ParseError {
                message,
                line,
                column,
            } => write!(f, "Parse error at {}:{}: {}", line, column, message),
            ZenError::TypeError {
                message,
                line,
                column,
            } => write!(f, "Type error at {}:{}: {}", line, column, message),
            ZenError::CodegenError { message } => write!(f, "Code generation error: {}", message),
            ZenError::IoError { message } => write!(f, "I/O error: {}", message),
        }
    }
}

impl std::error::Error for ZenError {}

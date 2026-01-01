pub mod ast;
pub mod cli;
pub mod codegen;
pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod typechecker;

use std::fmt;

#[derive(Debug)]
pub struct ZenError {
    message: String,
}

impl fmt::Display for ZenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {}", self.message)
    }
}

impl std::error::Error for ZenError {}

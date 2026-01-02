use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Keywords
    Let,
    Mut,
    Fn,
    Return,
    If,
    Else,
    For,
    While,
    Match,
    Struct,
    Const,
    Mod,
    Use,
    As,
    Pub,
    Crate,
    Super,
    Self_,
    True,
    False,
    Null,

    // Types
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Bool,
    Str,
    Char,
    Void,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,

    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    EqualEqual,
    NotEqual,
    Equal,
    Not,
    And,
    Or,

    ArrowLeft,
    ArrowRight,
    Dot,
    DoubleDot,
    DotDot,

    Pipe,
    Ampersand,
    AmpersandMut,
    Bang,
    Tilde,
    Question,
    Colon,
    DoubleColon,
    Semicolon,
    Comma,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    // Literals
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
    CharLiteral,
    Identifier,

    // Special
    EOF,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Token {
            kind,
            lexeme,
            line,
            column,
        }
    }

    pub fn eof(line: usize, column: usize) -> Self {
        Token::new(TokenType::EOF, String::new(), line, column)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: '{}' at {}:{}",
            self.kind, self.lexeme, self.line, self.column
        )
    }
}

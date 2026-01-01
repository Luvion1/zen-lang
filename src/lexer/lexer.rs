use std::iter::Peekable;
use std::str::Chars;

use crate::token::{Token, TokenType};

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().peekable(),
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.peek().is_some() {
            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }

        tokens.push(Token::eof(self.line, self.column));
        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        let ch = self.advance()?;

        match ch {
            // Skip whitespace and newlines
            ' ' | '\t' | '\r' => self.next_token(),
            '\n' => {
                self.line += 1;
                self.column = 1;
                self.next_token()
            }

            // Comments
            '/' => {
                if self.peek() == Some('/') {
                    // Single line comment - skip to end of line
                    while self.peek().is_some() && self.peek() != Some('\n') {
                        self.advance();
                    }
                    self.next_token()
                } else if self.peek() == Some('*') {
                    // Multi-line comment
                    self.advance(); // consume *
                    let start_line = self.line;
                    let start_col = self.column;
                    while self.peek().is_some() {
                        if self.advance() == Some('*') && self.peek() == Some('/') {
                            self.advance(); // consume /
                            return self.next_token();
                        }
                    }
                    Some(Token::new(
                        TokenType::Unknown,
                        "Unterminated multi-line comment".to_string(),
                        start_line,
                        start_col,
                    ))
                } else {
                    Some(Token::new(
                        TokenType::Slash,
                        "/".to_string(),
                        self.line,
                        self.column - 1,
                    ))
                }
            }

            // Operators
            '+' => Some(Token::new(
                TokenType::Plus,
                "+".to_string(),
                self.line,
                self.column - 1,
            )),
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    Some(Token::new(
                        TokenType::ArrowRight,
                        "->".to_string(),
                        self.line,
                        self.column - 2,
                    ))
                } else {
                    Some(Token::new(
                        TokenType::Minus,
                        "-".to_string(),
                        self.line,
                        self.column - 1,
                    ))
                }
            }
            '*' => Some(Token::new(
                TokenType::Star,
                "*".to_string(),
                self.line,
                self.column - 1,
            )),
            '%' => Some(Token::new(
                TokenType::Percent,
                "%".to_string(),
                self.line,
                self.column - 1,
            )),
            '^' => Some(Token::new(
                TokenType::Caret,
                "^".to_string(),
                self.line,
                self.column - 1,
            )),

            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(
                        TokenType::LessEqual,
                        "<=".to_string(),
                        self.line,
                        self.column - 2,
                    ))
                } else if self.peek() == Some('-') {
                    self.advance();
                    Some(Token::new(
                        TokenType::ArrowLeft,
                        "<-".to_string(),
                        self.line,
                        self.column - 2,
                    ))
                } else {
                    Some(Token::new(
                        TokenType::LessThan,
                        "<".to_string(),
                        self.line,
                        self.column - 1,
                    ))
                }
            }

            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(
                        TokenType::GreaterEqual,
                        ">=".to_string(),
                        self.line,
                        self.column - 2,
                    ))
                } else {
                    Some(Token::new(
                        TokenType::GreaterThan,
                        ">".to_string(),
                        self.line,
                        self.column - 1,
                    ))
                }
            }

            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(
                        TokenType::EqualEqual,
                        "==".to_string(),
                        self.line,
                        self.column - 2,
                    ))
                } else if self.peek() == Some('>') {
                    self.advance();
                    Some(Token::new(
                        TokenType::ArrowRight,
                        "=>".to_string(),
                        self.line,
                        self.column - 2,
                    ))
                } else {
                    Some(Token::new(
                        TokenType::Equal,
                        "=".to_string(),
                        self.line,
                        self.column - 1,
                    ))
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::new(
                        TokenType::NotEqual,
                        "!=".to_string(),
                        self.line,
                        self.column - 2,
                    ))
                } else {
                    Some(Token::new(
                        TokenType::Not,
                        "!".to_string(),
                        self.line,
                        self.column - 1,
                    ))
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    Some(Token::new(
                        TokenType::And,
                        "&&".to_string(),
                        self.line,
                        self.column - 2,
                    ))
                } else {
                    Some(Token::new(
                        TokenType::Ampersand,
                        "&".to_string(),
                        self.line,
                        self.column - 1,
                    ))
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    Some(Token::new(
                        TokenType::Or,
                        "||".to_string(),
                        self.line,
                        self.column - 2,
                    ))
                } else {
                    Some(Token::new(
                        TokenType::Pipe,
                        "|".to_string(),
                        self.line,
                        self.column - 1,
                    ))
                }
            }
            '~' => Some(Token::new(
                TokenType::Tilde,
                "~".to_string(),
                self.line,
                self.column - 1,
            )),
            '?' => Some(Token::new(
                TokenType::Question,
                "?".to_string(),
                self.line,
                self.column - 1,
            )),
            ':' => {
                if self.peek() == Some(':') {
                    self.advance();
                    Some(Token::new(
                        TokenType::DoubleColon,
                        "::".to_string(),
                        self.line,
                        self.column - 2,
                    ))
                } else {
                    Some(Token::new(
                        TokenType::Colon,
                        ":".to_string(),
                        self.line,
                        self.column - 1,
                    ))
                }
            }
            ';' => Some(Token::new(
                TokenType::Semicolon,
                ";".to_string(),
                self.line,
                self.column - 1,
            )),
            ',' => Some(Token::new(
                TokenType::Comma,
                ",".to_string(),
                self.line,
                self.column - 1,
            )),
            '.' => {
                if self.peek() == Some('.') {
                    self.advance();
                    Some(Token::new(
                        TokenType::DotDot,
                        "..".to_string(),
                        self.line,
                        self.column - 2,
                    ))
                } else {
                    Some(Token::new(
                        TokenType::Dot,
                        ".".to_string(),
                        self.line,
                        self.column - 1,
                    ))
                }
            }

            // Delimiters
            '(' => Some(Token::new(
                TokenType::LeftParen,
                "(".to_string(),
                self.line,
                self.column - 1,
            )),
            ')' => Some(Token::new(
                TokenType::RightParen,
                ")".to_string(),
                self.line,
                self.column - 1,
            )),
            '{' => Some(Token::new(
                TokenType::LeftBrace,
                "{".to_string(),
                self.line,
                self.column - 1,
            )),
            '}' => Some(Token::new(
                TokenType::RightBrace,
                "}".to_string(),
                self.line,
                self.column - 1,
            )),
            '[' => Some(Token::new(
                TokenType::LeftBracket,
                "[".to_string(),
                self.line,
                self.column - 1,
            )),
            ']' => Some(Token::new(
                TokenType::RightBracket,
                "]".to_string(),
                self.line,
                self.column - 1,
            )),

            // String literals
            '"' => self.string_literal(),

            // Character literals
            '\'' => self.char_literal(),

            // Numbers or identifiers
            '0'..='9' => self.number_literal(ch),
            'a'..='z' | 'A'..='Z' | '_' => self.identifier_or_keyword(ch),

            // Unknown
            _ => Some(Token::new(
                TokenType::Unknown,
                ch.to_string(),
                self.line,
                self.column - 1,
            )),
        }
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.next();
        if ch.is_some() {
            self.column += 1;
        }
        ch
    }

    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    fn string_literal(&mut self) -> Option<Token> {
        let start_line = self.line;
        let start_col = self.column - 1;
        let mut lexeme = String::new();
        lexeme.push('"');

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                lexeme.push('"');
                return Some(Token::new(
                    TokenType::StringLiteral,
                    lexeme,
                    start_line,
                    start_col,
                ));
            }
            if ch == '\\' {
                self.advance();
                lexeme.push(ch);
                if let Some(escaped) = self.advance() {
                    lexeme.push(escaped);
                }
            } else {
                self.advance();
                lexeme.push(ch);
            }
        }

        Some(Token::new(
            TokenType::Unknown,
            format!("Unterminated string: {}", lexeme),
            start_line,
            start_col,
        ))
    }

    fn char_literal(&mut self) -> Option<Token> {
        let start_line = self.line;
        let start_col = self.column - 1;
        let mut lexeme = String::new();
        lexeme.push('\'');

        if let Some(ch) = self.advance() {
            lexeme.push(ch);
            if ch == '\\' && self.peek().is_some() {
                if let Some(escaped) = self.advance() {
                    lexeme.push(escaped);
                }
            }
        }

        if self.advance() == Some('\'') {
            lexeme.push('\'');
            Some(Token::new(
                TokenType::CharLiteral,
                lexeme,
                start_line,
                start_col,
            ))
        } else {
            Some(Token::new(
                TokenType::Unknown,
                format!("Unterminated char: {}", lexeme),
                start_line,
                start_col,
            ))
        }
    }

    fn number_literal(&mut self, first: char) -> Option<Token> {
        let start_line = self.line;
        let start_col = self.column - 1;
        let mut lexeme = String::new();
        lexeme.push(first);

        let mut is_float = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
                lexeme.push(ch);
            } else if ch == '.' && !is_float {
                self.advance();
                lexeme.push(ch);
                is_float = true;
            } else if ch == '_' {
                self.advance();
                lexeme.push(ch);
            } else {
                break;
            }
        }

        // Check for float type suffix
        if self.peek() == Some('f') {
            self.advance();
            lexeme.push('f');
            if let Some(ch) = self.peek() {
                if ch == '3' || ch == '6' {
                    self.advance();
                    lexeme.push(ch);
                    if ch == '3' && self.peek() == Some('2') {
                        self.advance();
                        lexeme.push('2');
                    } else if ch == '6' && self.peek() == Some('4') {
                        self.advance();
                        lexeme.push('4');
                    }
                }
            }
        }

        // Check for integer type suffix
        if self.peek() == Some('u') || self.peek() == Some('i') {
            self.advance(); // Consume 'u' or 'i'
            if let Some(ch) = self.peek() {
                if ch == '8' || ch == '1' || ch == '3' || ch == '6' {
                    self.advance();
                    lexeme.push(ch);
                    if (ch == '1' || ch == '3') && self.peek() == Some('6') {
                        self.advance();
                        lexeme.push('6');
                        if ch == '1' && self.peek() == Some('2') {
                            self.advance();
                            lexeme.push('2');
                        }
                    }
                }
            }
        }

        if is_float {
            Some(Token::new(
                TokenType::FloatLiteral,
                lexeme,
                start_line,
                start_col,
            ))
        } else {
            Some(Token::new(
                TokenType::IntegerLiteral,
                lexeme,
                start_line,
                start_col,
            ))
        }
    }

    fn identifier_or_keyword(&mut self, first: char) -> Option<Token> {
        let start_line = self.line;
        let start_col = self.column - 1;
        let mut lexeme = String::new();
        lexeme.push(first);

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
                lexeme.push(ch);
            } else {
                break;
            }
        }

        let token_type = match lexeme.as_str() {
            "let" => TokenType::Let,
            "mut" => TokenType::Mut,
            "fn" => TokenType::Fn,
            "return" => TokenType::Return,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "match" => TokenType::Match,
            "struct" => TokenType::Struct,
            "const" => TokenType::Const,
            "mod" => TokenType::Mod,
            "use" => TokenType::Use,
            "pub" => TokenType::Pub,
            "crate" => TokenType::Crate,
            "super" => TokenType::Super,
            "self" => TokenType::Self_,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            "i8" => TokenType::Int8,
            "i16" => TokenType::Int16,
            "i32" => TokenType::Int32,
            "i64" => TokenType::Int64,
            "u8" => TokenType::UInt8,
            "u16" => TokenType::UInt16,
            "u32" => TokenType::UInt32,
            "u64" => TokenType::UInt64,
            "f32" => TokenType::Float32,
            "f64" => TokenType::Float64,
            "bool" => TokenType::Bool,
            "str" => TokenType::Str,
            "char" => TokenType::Char,
            "void" => TokenType::Void,
            _ => TokenType::Identifier,
        };

        Some(Token::new(token_type, lexeme, start_line, start_col))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let code = "let mut fn return if else for while";
        let lexer = Lexer::new(code);
        let mut l = lexer;
        let tokens = l.tokenize();

        assert_eq!(tokens[0].kind, TokenType::Let);
        assert_eq!(tokens[1].kind, TokenType::Mut);
        assert_eq!(tokens[2].kind, TokenType::Fn);
        assert_eq!(tokens[3].kind, TokenType::Return);
        assert_eq!(tokens[4].kind, TokenType::If);
        assert_eq!(tokens[5].kind, TokenType::Else);
        assert_eq!(tokens[6].kind, TokenType::For);
        assert_eq!(tokens[7].kind, TokenType::While);
    }

    #[test]
    fn test_operators() {
        let code = "+ - * / % ^ < > <= >= == != = ! && ||";
        let lexer = Lexer::new(code);
        let mut l = lexer;
        let tokens = l.tokenize();

        assert_eq!(tokens[0].kind, TokenType::Plus);
        assert_eq!(tokens[1].kind, TokenType::Minus);
        assert_eq!(tokens[2].kind, TokenType::Star);
        assert_eq!(tokens[3].kind, TokenType::Slash);
        assert_eq!(tokens[4].kind, TokenType::Percent);
        assert_eq!(tokens[5].kind, TokenType::Caret);
    }

    #[test]
    fn test_ownership_transfer() {
        let code = "<-";
        let lexer = Lexer::new(code);
        let mut l = lexer;
        let tokens = l.tokenize();

        assert_eq!(tokens[0].kind, TokenType::ArrowLeft);
    }

    #[test]
    fn test_literals() {
        let code = "42 3.14 \"hello\" 'c'";
        let lexer = Lexer::new(code);
        let mut l = lexer;
        let tokens = l.tokenize();

        assert_eq!(tokens[0].kind, TokenType::IntegerLiteral);
        assert_eq!(tokens[0].lexeme, "42");
        assert_eq!(tokens[1].kind, TokenType::FloatLiteral);
        assert_eq!(tokens[1].lexeme, "3.14");
        assert_eq!(tokens[2].kind, TokenType::StringLiteral);
        assert_eq!(tokens[2].lexeme, "\"hello\"");
        assert_eq!(tokens[3].kind, TokenType::CharLiteral);
        assert_eq!(tokens[3].lexeme, "'c'");
    }

    #[test]
    fn test_identifiers() {
        let code = "my_variable function_name _private";
        let lexer = Lexer::new(code);
        let mut l = lexer;
        let tokens = l.tokenize();

        assert_eq!(tokens[0].kind, TokenType::Identifier);
        assert_eq!(tokens[0].lexeme, "my_variable");
        assert_eq!(tokens[1].kind, TokenType::Identifier);
        assert_eq!(tokens[1].lexeme, "function_name");
        assert_eq!(tokens[2].kind, TokenType::Identifier);
        assert_eq!(tokens[2].lexeme, "_private");
    }

    #[test]
    fn test_comments() {
        let code = "let x = 10 // this is a comment\nlet y = 20";
        let lexer = Lexer::new(code);
        let mut l = lexer;
        let tokens = l.tokenize();

        assert_eq!(tokens[0].kind, TokenType::Let);
        assert_eq!(tokens[1].kind, TokenType::Identifier);
        assert_eq!(tokens[2].kind, TokenType::Equal);
        assert_eq!(tokens[3].kind, TokenType::IntegerLiteral);
        assert_eq!(tokens[4].kind, TokenType::Let);
    }

    #[test]
    fn test_types() {
        let code = "i32 f64 bool str char void";
        let lexer = Lexer::new(code);
        let mut l = lexer;
        let tokens = l.tokenize();

        assert_eq!(tokens[0].kind, TokenType::Int32);
        assert_eq!(tokens[1].kind, TokenType::Float64);
        assert_eq!(tokens[2].kind, TokenType::Bool);
        assert_eq!(tokens[3].kind, TokenType::Str);
        assert_eq!(tokens[4].kind, TokenType::Char);
        assert_eq!(tokens[5].kind, TokenType::Void);
    }
}

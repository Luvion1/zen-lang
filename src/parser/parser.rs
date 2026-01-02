use crate::ast::expr::Expr;
use crate::ast::program::Program;
use crate::ast::stmt::Stmt;
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    // Enhanced error tracking
    errors: Vec<String>,
    panic_mode: bool,
    had_error: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { 
            tokens, 
            current: 0,
            errors: Vec::new(),
            panic_mode: false,
            had_error: false,
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut program = Program::new();

        while !self.is_at_end() {
            if self.panic_mode {
                self.synchronize();
            }

            match self.declaration() {
                Ok(Some(stmt)) => program.add_statement(stmt),
                Ok(None) => continue,
                Err(e) => {
                    self.report_error(e);
                    self.synchronize();
                }
            }
        }

        if self.had_error {
            let error_summary = format!("Parsing failed with {} errors:\n{}", 
                                      self.errors.len(), 
                                      self.errors.join("\n"));
            Err(error_summary)
        } else {
            Ok(program)
        }
    }

    fn report_error(&mut self, message: String) {
        if self.panic_mode {
            return;
        }
        
        self.panic_mode = true;
        self.had_error = true;
        
        let current_token = self.peek();
        let error_msg = format!("Error at line {}, column {}: {} (token: {:?})", 
                               current_token.line, 
                               current_token.column, 
                               message,
                               current_token.kind);
        
        self.errors.push(error_msg);
        eprintln!("Parse error: {}", message);
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        
        while !self.is_at_end() {
            if self.previous().kind == TokenType::Semicolon {
                return;
            }

            match self.peek().kind {
                TokenType::Fn | TokenType::Let | TokenType::If | 
                TokenType::While | TokenType::For | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn declaration(&mut self) -> Result<Option<Stmt>, String> {
        if self.check(TokenType::Fn) {
            return Ok(Some(self.function_declaration()?));
        }
        if self.check(TokenType::Struct) {
            return Ok(Some(self.struct_declaration()?));
        }
        if self.check(TokenType::Let) || self.check(TokenType::Mut) {
            return Ok(Some(self.variable_declaration()?));
        }
        self.statement().map(Some)
    }

    fn function_declaration(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::Fn, "Expected 'fn' keyword")?;
        let name = self.consume_identifier()?;

        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;
        let params = self.parameters()?;
        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;

        self.consume(TokenType::ArrowRight, "Expected '->' after parameters")?;
        let return_type = self.type_annotation()?;

        let body = self.block()?;

        Ok(Stmt::FunctionDecl {
            name,
            params,
            return_type,
            body,
            is_public: false, // For now, all functions are private
            token: self.previous().clone(),
        })
    }

    fn struct_declaration(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::Struct, "Expected 'struct' keyword")?;
        let name = self.consume_identifier()?;

        self.consume(TokenType::LeftBrace, "Expected '{' after struct name")?;

        let mut fields = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let field_name = self.consume_identifier()?;
            self.consume(TokenType::Colon, "Expected ':' after field name")?;
            let field_type = self.type_annotation()?;

            fields.push((field_name, field_type));

            if !self.match_token(TokenType::Comma) && !self.check(TokenType::RightBrace) {
                return Err("Expected ',' or '}' after field".to_string());
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after struct fields")?;

        Ok(Stmt::StructDecl {
            name,
            fields,
            is_public: false, // For now, all structs are private
            token: self.previous().clone(),
        })
    }

    fn variable_declaration(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::Let, "Expected 'let' keyword")?;
        let is_mutable = self.match_token(TokenType::Mut);
        let name = self.consume_identifier()?;

        let type_annotation = if self.match_token(TokenType::Colon) {
            Some(self.type_annotation()?)
        } else {
            None
        };

        let initializer = if self.match_token(TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        Ok(Stmt::VariableDecl {
            name,
            type_annotation,
            initializer,
            is_mutable,
            token: self.previous().clone(),
        })
    }

    fn parameters(&mut self) -> Result<Vec<(String, String)>, String> {
        let mut params = Vec::new();

        if !self.check(TokenType::RightParen) {
            params.push(self.param()?);

            while self.match_token(TokenType::Comma) {
                params.push(self.param()?);
            }
        }

        Ok(params)
    }

    fn param(&mut self) -> Result<(String, String), String> {
        let name = self.consume_identifier()?;
        self.consume(TokenType::Colon, "Expected ':' after parameter name")?;
        let type_annotation = self.type_annotation()?;
        Ok((name, type_annotation))
    }

    fn type_annotation(&mut self) -> Result<String, String> {
        // Check for array type: [ElementType; Size] or [ElementType]
        if self.match_token(TokenType::LeftBracket) {
            // Parse element type - can be any valid type
            let element_type = self.parse_type_name()?;
            let mut array_spec = format!("[{}", element_type);

            if self.match_token(TokenType::Semicolon) {
                let size_token = self.consume_identifier()?; // Get the integer literal as string
                array_spec.push_str(&format!("; {}", size_token));
            }

            self.consume(TokenType::RightBracket, "Expected ']' after array type")?;
            return Ok(array_spec);
        }

        // Regular type identifier
        self.parse_type_name()
    }

    fn parse_type_name(&mut self) -> Result<String, String> {
        let token = self.advance();

        // Handle built-in types
        match token.kind {
            TokenType::Int8 | TokenType::Int16 | TokenType::Int32 | TokenType::Int64 |
            TokenType::UInt8 | TokenType::UInt16 | TokenType::UInt32 | TokenType::UInt64 |
            TokenType::Float32 | TokenType::Float64 | TokenType::Bool |
            TokenType::Str | TokenType::Char | TokenType::Void => {
                Ok(token.lexeme.clone())
            }
            TokenType::Identifier => {
                Ok(token.lexeme.clone())
            }
            _ => Err(format!("Expected type name, found {:?}", token.kind)),
        }
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.check(TokenType::Return) {
            return self.return_statement();
        }
        if self.check(TokenType::LeftBrace) {
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }
        if self.check(TokenType::If) {
            return self.if_statement();
        }
        if self.check(TokenType::While) {
            return self.while_statement();
        }
        if self.check(TokenType::For) {
            return self.for_statement();
        }
        if self.check(TokenType::Match) {
            return self.match_statement();
        }
        if self.check(TokenType::Use) {
            return self.use_statement();
        }

        if self.check(TokenType::Let) {
            return self.variable_declaration();
        }

        self.expression_statement()
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;

        if let Expr::BinaryOp { op, left, right } = &expr {
            if op.kind == TokenType::Equal {
                if let Expr::Identifier { .. } = left.as_ref() {
                    return Ok(Stmt::Assignment {
                        target: *left.clone(),
                        value: *right.clone(),
                        token: op.clone(),
                    });
                }
            }
        }

        self.match_token(TokenType::Semicolon);
        Ok(Stmt::ExprStmt { expr })
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::Return, "Expected 'return' keyword")?;
        let value = if !self.check(TokenType::Semicolon) && !self.check(TokenType::RightBrace) {
            Some(self.expression()?)
        } else {
            None
        };

        Ok(Stmt::Return {
            value,
            token: self.previous().clone(),
        })
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        let if_token = self.peek().clone();
        self.consume(TokenType::If, "Expected 'if' keyword")?;
        let condition = self.expression()?;
        let then_branch = self.block()?;
        
        // Parse all else if branches
        let mut else_if_branches = Vec::new();
        
        while self.check(TokenType::Else) && self.check_ahead(1, TokenType::If) {
            self.advance(); // consume 'else'
            let else_if_token = self.peek().clone();
            self.consume(TokenType::If, "Expected 'if' after 'else'")?;
            let else_if_condition = self.expression()?;
            let else_if_body = self.block()?;
            
            else_if_branches.push(crate::ast::stmt::ElseIfBranch {
                condition: else_if_condition,
                body: else_if_body,
                token: else_if_token,
            });
        }
        
        // Parse final else branch if present
        let else_branch = if self.match_token(TokenType::Else) {
            Some(self.block()?)
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
            token: if_token,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::While, "Expected 'while' keyword")?;
        let condition = self.expression()?;
        let body = self.block()?;

        Ok(Stmt::While {
            condition,
            body,
            token: self.previous().clone(),
        })
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::For, "Expected 'for' keyword")?;
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'")?;

        let init = if self.match_token(TokenType::Semicolon) {
            None
        } else if self.check(TokenType::Let) {
            Some(Box::new(self.variable_declaration()?))
        } else {
            let is_mutable = self.match_token(TokenType::Mut);
            let name = self.consume_identifier()?;
            self.consume(TokenType::Equal, "Expected '=' in for init")?;
            let value = self.expression()?;
            Some(Box::new(Stmt::VariableDecl {
                name,
                type_annotation: None,
                initializer: Some(value),
                is_mutable,
                token: self.previous().clone(),
            }))
        };

        self.consume(TokenType::Semicolon, "Expected ';' after for init")?;

        let condition = if self.match_token(TokenType::Semicolon) {
            None
        } else {
            let cond = self.expression()?;
            self.consume(TokenType::Semicolon, "Expected ';' after for condition")?;
            Some(cond)
        };

        let increment = if self.check(TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(TokenType::RightParen, "Expected ')' after for clauses")?;
        let body = self.block()?;

        Ok(Stmt::For {
            init,
            condition,
            increment,
            body,
            token: self.previous().clone(),
        })
    }

    fn match_statement(&mut self) -> Result<Stmt, String> {
        let match_token = self.advance();
        let value = self.expression()?;

        self.consume(TokenType::LeftBrace, "Expected '{' after match value")?;

        let mut arms = Vec::new();
        let mut default = None;

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let pattern = self.expression()?;
            self.consume(TokenType::ArrowRight, "Expected '=>' after match pattern")?;

            if let Expr::Identifier { name, .. } = &pattern {
                if name == "_" {
                    let stmt = self.statement()?;
                    let body = if let Stmt::Block { statements } = stmt {
                        statements
                    } else {
                        vec![stmt]
                    };
                    default = Some(body);
                } else {
                    let stmt = self.statement()?;
                    let body = if let Stmt::Block { statements } = stmt {
                        statements
                    } else {
                        vec![stmt]
                    };
                    arms.push((pattern, body));
                }
            } else {
                let stmt = self.statement()?;
                let body = if let Stmt::Block { statements } = stmt {
                    statements
                } else {
                    vec![stmt]
                };
                arms.push((pattern, body));
            }

            self.match_token(TokenType::Comma);
        }

        self.consume(TokenType::RightBrace, "Expected '}' to close match")?;

        Ok(Stmt::Match {
            value,
            arms,
            default,
            token: match_token,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        self.consume(TokenType::LeftBrace, "Expected '{'")?;
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.declaration()? {
                statements.push(stmt);
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}'")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.logical_or()?;

        if self.match_token(TokenType::Equal) {
            let equal_token = self.previous().clone();
            let value = self.assignment()?;
            if let Expr::Identifier { .. } = expr {
                return Ok(Expr::BinaryOp {
                    left: Box::new(expr),
                    op: equal_token,
                    right: Box::new(value),
                });
            }
            return Err("Invalid assignment target".to_string());
        }

        Ok(expr)
    }

    fn logical_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.logical_and()?;

        while self.match_token(TokenType::Or) {
            let op = self.previous().clone();
            let right = self.logical_and()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.match_token(TokenType::And) {
            let op = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_token(TokenType::EqualEqual) || self.match_token(TokenType::NotEqual) {
            let op = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_token(TokenType::GreaterThan)
            || self.match_token(TokenType::GreaterEqual)
            || self.match_token(TokenType::LessThan)
            || self.match_token(TokenType::LessEqual)
        {
            let op = self.previous().clone();
            let right = self.term()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_token(TokenType::Plus) || self.match_token(TokenType::Minus) {
            let op = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_token(TokenType::Star)
            || self.match_token(TokenType::Slash)
            || self.match_token(TokenType::Percent)
        {
            let op = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_token(TokenType::Not)
            || self.match_token(TokenType::Minus)
            || self.match_token(TokenType::ArrowLeft)
            || self.match_token(TokenType::Ampersand)
            || self.match_token(TokenType::AmpersandMut)
        {
            let op = self.previous().clone();
            let right = self.unary()?;

            match op.kind {
                TokenType::ArrowLeft => Ok(Expr::OwnershipTransfer {
                    expr: Box::new(right),
                    token: op,
                }),
                TokenType::Ampersand => Ok(Expr::Borrow {
                    expr: Box::new(right),
                    is_mutable: false,
                    token: op,
                }),
                TokenType::AmpersandMut => Ok(Expr::Borrow {
                    expr: Box::new(right),
                    is_mutable: true,
                    token: op,
                }),
                _ => Ok(Expr::UnaryOp {
                    op,
                    operand: Box::new(right),
                }),
            }
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(TokenType::LeftParen) {
                // Function call: expr(args)
                let mut args = Vec::new();

                if !self.check(TokenType::RightParen) {
                    args.push(self.expression()?);
                    while self.match_token(TokenType::Comma) {
                        args.push(self.expression()?);
                    }
                }

                self.consume(TokenType::RightParen, "Expected ')' after arguments")?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                    token: self.previous().clone(),
                };
            } else if self.match_token(TokenType::LeftBracket) {
                // Array access: expr[index]
                let index = self.expression()?;
                self.consume(TokenType::RightBracket, "Expected ']' after array index")?;
                expr = Expr::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                    token: self.previous().clone(),
                };
            } else if self.match_token(TokenType::Dot) {
                // Member access: expr.field
                let field = self.consume_identifier()?;
                expr = Expr::FieldAccess {
                    object: Box::new(expr),
                    field,
                    token: self.previous().clone(),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.match_token(TokenType::True) {
            return Ok(Expr::BooleanLiteral {
                value: true,
                token: self.previous().clone(),
            });
        }

        if self.match_token(TokenType::False) {
            return Ok(Expr::BooleanLiteral {
                value: false,
                token: self.previous().clone(),
            });
        }

        if self.match_token(TokenType::Null) {
            return Ok(Expr::Identifier {
                name: "null".to_string(),
                token: self.previous().clone(),
            });
        }

        if let Some(number) = self.match_number() {
            return Ok(number);
        }

        if let Some(string_lit) = self.match_string() {
            return Ok(string_lit);
        }

        if let Some(char_lit) = self.match_char() {
            return Ok(char_lit);
        }

        if self.match_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }

        if self.check(TokenType::Identifier) {
            let token = self.advance();
            let name = token.lexeme.clone();

            // Check for module access: module::item
            if self.match_token(TokenType::DoubleColon) {
                let item_name = self.consume_identifier()?;
                return Ok(Expr::ModuleAccess {
                    module: name,
                    item: item_name,
                    token,
                });
            }

            // Check for struct literal: StructName { ... }
            // Use lookahead to distinguish from other uses of '{'
            if self.check(TokenType::LeftBrace) && self.is_struct_literal_context() {
                self.advance(); // consume '{'
                let fields = self.struct_literal_fields()?;
                self.consume(TokenType::RightBrace, "Expected '}' after struct literal fields")?;
                return Ok(Expr::StructLiteral {
                    struct_name: name,
                    fields,
                    token,
                });
            }

            return Ok(Expr::Identifier {
                name,
                token,
            });
        }

        Err(format!("Unexpected token: {:?}", self.peek()))
    }

    fn is_struct_literal_context(&mut self) -> bool {
        // Look ahead to see if this looks like a struct literal
        // Struct literal: { field: value, ... } or { }
        let lookahead = 1;
        
        // Skip whitespace/comments if any (simplified check)
        while lookahead < self.tokens.len() - self.current {
            let token = &self.tokens[self.current + lookahead];
            match token.kind {
                TokenType::RightBrace => return true, // Empty struct literal
                TokenType::Identifier => {
                    // Check if followed by ':' (field: value pattern)
                    if lookahead + 1 < self.tokens.len() - self.current {
                        let next_token = &self.tokens[self.current + lookahead + 1];
                        return next_token.kind == TokenType::Colon;
                    }
                    return false;
                }
                _ => return false, // Not a struct literal pattern
            }
        }
        false
    }

    fn struct_literal_fields(&mut self) -> Result<Vec<(String, Expr)>, String> {
        let mut fields = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let field_name = self.consume_identifier()?;
            self.consume(TokenType::Colon, "Expected ':' after field name")?;
            let field_value = self.expression()?;

            fields.push((field_name, field_value));

            if !self.match_token(TokenType::Comma) && !self.check(TokenType::RightBrace) {
                return Err("Expected ',' or '}' after field".to_string());
            }
        }

        Ok(fields)
    }

    fn match_number(&mut self) -> Option<Expr> {
        if self.check(TokenType::IntegerLiteral) {
            let token = self.advance();
            return Some(Expr::IntegerLiteral {
                value: token.lexeme.clone(),
                token,
            });
        }

        if self.check(TokenType::FloatLiteral) {
            let token = self.advance();
            if let Ok(value) = token.lexeme.parse::<f64>() {
                return Some(Expr::FloatLiteral { value, token });
            }
        }

        None
    }

    fn match_string(&mut self) -> Option<Expr> {
        if self.check(TokenType::StringLiteral) {
            let token = self.advance();
            if token.lexeme.len() < 2 {
                return None; // Invalid string literal
            }
            let value = token.lexeme[1..token.lexeme.len() - 1].to_string();
            
            // Check if string contains interpolation
            if value.contains('{') && value.contains('}') {
                let parts = self.parse_interpolated_string(&value);
                return Some(Expr::InterpolatedString { 
                    parts, 
                    token 
                });
            }
            
            return Some(Expr::StringLiteral { value, token });
        }
        None
    }

    fn match_char(&mut self) -> Option<Expr> {
        if self.check(TokenType::CharLiteral) {
            let token = self.advance();
            if token.lexeme.len() != 3 || !token.lexeme.starts_with('\'') || !token.lexeme.ends_with('\'') {
                return None; // Invalid char literal format
            }
            let value = token.lexeme.chars().nth(1).unwrap_or('\0');
            return Some(Expr::CharLiteral { value, token });
        }
        None
    }

    fn parse_interpolated_string(&self, value: &str) -> Vec<crate::ast::expr::StringPart> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = value.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Save any text before the variable
                if !current.is_empty() {
                    parts.push(crate::ast::expr::StringPart::Text(current.clone()));
                    current.clear();
                }
                
                // Extract variable name or expression
                let mut expr_content = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume '}'
                        break;
                    }
                    expr_content.push(chars.next().unwrap());
                }
                
                if !expr_content.is_empty() {
                    // Check if it's a function call (contains parentheses)
                    if expr_content.contains('(') && expr_content.contains(')') {
                        parts.push(crate::ast::expr::StringPart::Expression(expr_content));
                    } else {
                        parts.push(crate::ast::expr::StringPart::Variable(expr_content));
                    }
                }
            } else {
                current.push(ch);
            }
        }
        
        // Add remaining text
        if !current.is_empty() {
            parts.push(crate::ast::expr::StringPart::Text(current));
        }
        
        parts
    }

    fn consume_identifier(&mut self) -> Result<String, String> {
        if self.check(TokenType::Identifier) {
            return Ok(self.advance().lexeme);
        }
        Err(format!("Expected identifier, got {:?}", self.peek()))
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<(), String> {
        if self.check(token_type.clone()) {
            self.advance();
            return Ok(());
        }
        
        let current = self.peek();
        let detailed_error = format!(
            "{} at line {}, column {}. Expected {:?}, but found {:?} '{}'",
            message, 
            current.line, 
            current.column,
            token_type,
            current.kind,
            current.lexeme
        );
        
        // Enhanced error context
        let context = self.get_error_context();
        let full_error = if !context.is_empty() {
            format!("{}\nContext: {}", detailed_error, context)
        } else {
            detailed_error
        };
        
        Err(full_error)
    }

    fn get_error_context(&self) -> String {
        // Provide context around the error location
        let mut context = String::new();
        let start = if self.current >= 2 { self.current - 2 } else { 0 };
        let end = std::cmp::min(self.current + 3, self.tokens.len());
        
        for i in start..end {
            if i < self.tokens.len() {
                let marker = if i == self.current { " >>> " } else { "     " };
                context.push_str(&format!("{}Token {}: {:?} '{}'\n", 
                                        marker, i, self.tokens[i].kind, self.tokens[i].lexeme));
            }
        }
        
        context
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            return true;
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().kind == token_type
    }

    fn check_ahead(&self, offset: usize, token_type: TokenType) -> bool {
        let index = self.current + offset;
        if index >= self.tokens.len() {
            return false;
        }
        self.tokens[index].kind == token_type
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenType::EOF
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn use_statement(&mut self) -> Result<Stmt, String> {
        let token = self.advance(); // consume 'use'
        let mut path = Vec::new();

        // Parse path like: crate::module::item or module::*
        loop {
            if self.check(TokenType::Identifier) || self.check(TokenType::Crate) || self.check(TokenType::Super) || self.check(TokenType::Self_) {
                path.push(self.advance().lexeme);
            } else if self.check(TokenType::Star) {
                // Handle wildcard import: use module::*;
                path.push(self.advance().lexeme);
                break;
            } else {
                return Err(format!("Expected identifier or '*' in use path, got {:?}", self.peek()));
            }

            if !self.match_token(TokenType::DoubleColon) {
                break;
            }
        }

        let alias = if self.match_token(TokenType::As) {
            Some(self.consume_identifier()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expected ';' after use statement")?;
        Ok(Stmt::Use { path, alias, token })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let code = "fn main() -> i32 { return 0 }";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing should succeed");

        let program = result.expect("Failed to parse program");
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_function_with_params() {
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Parsing function with params should succeed"
        );
    }

    #[test]
    fn test_variable_declaration() {
        let code = "let x = 10";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Parsing variable declaration should succeed"
        );
    }

    #[test]
    fn test_mutable_variable() {
        let code = "let mut counter = 0";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing mutable variable should succeed");
    }

    #[test]
    fn test_variable_with_type() {
        let code = "let age: i32 = 25";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing variable with type should succeed");
    }

    #[test]
    fn test_if_statement() {
        let code = "if x > 5 { println(\"Big\") }";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing if statement should succeed");
    }

    #[test]
    fn test_if_else_statement() {
        let code = "if x > 5 { println(\"Big\") } else { println(\"Small\") }";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing if-else statement should succeed");
    }

    #[test]
    fn test_while_loop() {
        let code = "while counter < 10 { counter = counter + 1 }";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing while loop should succeed");
    }

    #[test]
    fn test_c_style_for_loop() {
        let code = "for (i = 0; i < 10; i = i + 1) { println(i) }";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing C-style for loop should succeed");
    }

    #[test]
    fn test_match_statement() {
        let code = "match value { 1 => println(\"One\"), _ => println(\"Other\") }";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing match statement should succeed");
    }

    #[test]
    fn test_function_call() {
        let code = "println(\"Hello\")";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing function call should succeed");
    }

    #[test]
    fn test_assignment() {
        let code = "x = 10";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing assignment should succeed");
    }

    #[test]
    fn test_binary_expressions() {
        let code = "x = 10 + 20 * 3";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing binary expressions should succeed");
    }

    #[test]
    fn test_comparison_expressions() {
        let code = "x > 5 && y < 10";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Parsing comparison expressions should succeed"
        );
    }

    #[test]
    fn test_complex_program() {
        let code = r#"
fn main() -> i32 {
    let x = 10
    if x > 5 {
        println("Big")
    } else {
        println("Small")
    }
    return 0
}
"#;
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing complex program should work");

        let program = result.expect("Failed to parse complex program");
        assert!(!program.statements.is_empty());
    }

    #[test]
    fn test_multiple_declarations() {
        let code =
            "fn add(a: i32, b: i32) -> i32 { a + b } fn main() -> i32 { let x = 10 return 0 }";
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok());

        let program = result.expect("Failed to parse multiple declarations");
        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn test_else_if_single() {
        let code = r#"
fn main() -> i32 {
    let x = 10
    if x > 15 {
        println("High")
    } else if x > 5 {
        println("Medium")
    } else {
        println("Low")
    }
    return 0
}
"#;
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing single else if should succeed");

        let program = result.expect("Failed to parse else if");
        assert_eq!(program.statements.len(), 1);
        
        if let Stmt::FunctionDecl { body, .. } = &program.statements[0] {
            if let Stmt::If { else_if_branches, .. } = &body[1] {
                assert_eq!(else_if_branches.len(), 1, "Should have exactly one else if branch");
            } else {
                panic!("Expected If statement");
            }
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_else_if_multiple() {
        let code = r#"
fn main() -> i32 {
    let score = 85
    if score >= 90 {
        println("A")
    } else if score >= 80 {
        println("B")
    } else if score >= 70 {
        println("C")
    } else if score >= 60 {
        println("D")
    } else {
        println("F")
    }
    return 0
}
"#;
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing multiple else if should succeed");

        let program = result.expect("Failed to parse multiple else if");
        assert_eq!(program.statements.len(), 1);
        
        if let Stmt::FunctionDecl { body, .. } = &program.statements[0] {
            if let Stmt::If { else_if_branches, else_branch, .. } = &body[1] {
                assert_eq!(else_if_branches.len(), 3, "Should have exactly three else if branches");
                assert!(else_branch.is_some(), "Should have final else branch");
            } else {
                panic!("Expected If statement");
            }
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_else_if_no_final_else() {
        let code = r#"
fn main() -> i32 {
    let x = 10
    if x > 20 {
        println("Very high")
    } else if x > 10 {
        println("High")
    } else if x > 5 {
        println("Medium")
    }
    return 0
}
"#;
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing else if without final else should succeed");

        let program = result.expect("Failed to parse else if without final else");
        assert_eq!(program.statements.len(), 1);
        
        if let Stmt::FunctionDecl { body, .. } = &program.statements[0] {
            if let Stmt::If { else_if_branches, else_branch, .. } = &body[1] {
                assert_eq!(else_if_branches.len(), 2, "Should have exactly two else if branches");
                assert!(else_branch.is_none(), "Should not have final else branch");
            } else {
                panic!("Expected If statement");
            }
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_nested_else_if() {
        let code = r#"
fn main() -> i32 {
    let x = 10
    let y = 20
    if x > 15 {
        if y > 25 {
            println("Both high")
        } else if y > 15 {
            println("Y medium, X high")
        } else {
            println("Y low, X high")
        }
    } else if x > 5 {
        println("X medium")
    } else {
        println("X low")
    }
    return 0
}
"#;
        let mut lexer = crate::lexer::lexer::Lexer::new(code);
        let mut parser = Parser::new(lexer.tokenize());

        let result = parser.parse();
        assert!(result.is_ok(), "Parsing nested else if should succeed");

        let program = result.expect("Failed to parse nested else if");
        assert_eq!(program.statements.len(), 1);
    }
}

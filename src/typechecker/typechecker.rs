use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use crate::token::TokenType;

struct TypeInfo {
    name: String,
    is_mutable: bool,
}

pub struct TypeChecker {
    variables: std::collections::HashMap<String, TypeInfo>,
    functions: std::collections::HashMap<String, (Vec<(String, String)>, String)>,
    current_function: Option<String>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            variables: std::collections::HashMap::new(),
            functions: std::collections::HashMap::new(),
            current_function: None,
        }
    }

    pub fn check(&mut self, program: &crate::ast::program::Program) -> Result<(), String> {
        for stmt in &program.statements {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::VariableDecl {
                name,
                type_annotation,
                initializer,
                is_mutable,
                token,
            } => {
                let var_type = if let Some(t) = type_annotation {
                    if !self.is_valid_type(t) {
                        return Err(format!(
                            "Invalid type '{}':{}: '{}' is not a valid type",
                            token.line, token.column, t
                        ));
                    }
                    t.clone()
                } else if let Some(init) = initializer {
                    self.check_expression(init)?
                } else {
                    "i32".to_string()
                };

                if let Some(ref init) = initializer {
                    let init_type = self.check_expression(init)?;
                    if var_type != init_type {
                        return Err(format!(
                            "Type mismatch in variable declaration at {}:{}: expected '{}' but initializer is '{}'",
                            token.line, token.column, var_type, init_type
                        ));
                    }
                }

                self.variables.insert(
                    name.clone(),
                    TypeInfo {
                        name: var_type,
                        is_mutable: *is_mutable,
                    },
                );
            }

            Stmt::Assignment { target, value, .. } => {
                let value_type = self.check_expression(value)?;

                if let Expr::Identifier { name, token, .. } = target {
                    if let Some(var_info) = self.variables.get(name) {
                        if !var_info.is_mutable {
                            return Err(format!(
                                "Cannot assign to immutable variable '{}' at {}:{}",
                                name, token.line, token.column
                            ));
                        }

                        if var_info.name != value_type {
                            return Err(format!(
                                "Type mismatch in assignment at {}:{}: variable '{}' is '{}' but got '{}'",
                                token.line, token.column, name, var_info.name, value_type
                            ));
                        }
                    } else {
                        return Err(format!(
                            "Undefined variable '{}' at {}:{}",
                            name, token.line, token.column
                        ));
                    }
                } else {
                    return Err("Can only assign to identifiers".to_string());
                }
            }

            Stmt::FunctionDecl {
                name,
                params,
                return_type,
                body,
                token,
            } => {
                if self.functions.contains_key(name) {
                    return Err(format!(
                        "Function '{}' already defined at {}:{}",
                        name, token.line, token.column
                    ));
                }

                let old_function = self.current_function.clone();
                self.current_function = Some(name.clone());

                let old_vars = std::mem::take(&mut self.variables);

                for (param_name, param_type) in params {
                    self.variables.insert(
                        param_name.clone(),
                        TypeInfo {
                            name: param_type.clone(),
                            is_mutable: false,
                        },
                    );
                }

                for stmt in body {
                    self.check_statement(stmt)?;
                }

                self.variables = old_vars;
                self.current_function = old_function;

                self.functions
                    .insert(name.clone(), (params.clone(), return_type.clone()));
            }

            Stmt::Return { value, token } => {
                let current_fn = self
                    .current_function
                    .as_ref()
                    .ok_or("Return statement outside function")?;

                let return_type = if let Some(ref_val) = value {
                    self.check_expression(ref_val)?
                } else {
                    "void".to_string()
                };

                let expected_return = self
                    .functions
                    .get(current_fn)
                    .map(|(_, rt)| rt.clone())
                    .unwrap_or("void".to_string());

                if return_type != expected_return && expected_return != "void" {
                    return Err(format!(
                        "Return type mismatch at {}:{}: expected '{}' but got '{}'",
                        token.line, token.column, expected_return, return_type
                    ));
                }
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_type = self.check_expression(condition)?;
                if cond_type != "bool" {
                    return Err(format!("If condition must be bool, got '{}'", cond_type));
                }

                for stmt in then_branch {
                    self.check_statement(stmt)?;
                }

                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.check_statement(stmt)?;
                    }
                }
            }

            Stmt::While {
                condition, body, ..
            } => {
                let cond_type = self.check_expression(condition)?;
                if cond_type != "bool" {
                    return Err("While condition must be bool".to_string());
                }

                for stmt in body {
                    self.check_statement(stmt)?;
                }
            }

            Stmt::For {
                init,
                condition,
                increment,
                body,
                ..
            } => {
                let mut loop_vars: Vec<String> = Vec::new();

                if let Some(init_stmt) = init {
                    if let Stmt::VariableDecl { name, .. } = init_stmt.as_ref() {
                        loop_vars.push(name.clone());
                    }
                    self.check_statement(init_stmt)?;
                }

                if let Some(cond_expr) = condition {
                    let cond_type = self.check_expression(cond_expr)?;
                    if cond_type != "bool" {
                        return Err("For condition must be bool".to_string());
                    }
                }

                if let Some(inc_expr) = increment {
                    self.check_expression(inc_expr)?;
                }

                for stmt in body {
                    self.check_statement(stmt)?;
                }

                for var_name in loop_vars {
                    self.variables.remove(&var_name);
                }
            }

            Stmt::Match {
                value,
                arms,
                default,
                ..
            } => {
                let value_type = self.check_expression(value)?;

                for (pattern, body) in arms {
                    let pattern_type = self.check_expression(pattern)?;
                    if pattern_type != value_type {
                        return Err(format!(
                            "Match pattern type mismatch: expected '{}' but got '{}'",
                            value_type, pattern_type
                        ));
                    }
                    for stmt in body {
                        self.check_statement(stmt)?;
                    }
                }

                if let Some(default_body) = default {
                    for stmt in default_body {
                        self.check_statement(stmt)?;
                    }
                }
            }

            Stmt::ExprStmt { expr } => {
                self.check_expression(expr)?;
            }

            Stmt::Block { statements } => {
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
            }
        }

        Ok(())
    }

    fn check_expression(&self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::IntegerLiteral { .. } => Ok("i32".to_string()),
            Expr::FloatLiteral { .. } => Ok("f64".to_string()),
            Expr::StringLiteral { .. } => Ok("str".to_string()),
            Expr::CharLiteral { .. } => Ok("char".to_string()),
            Expr::BooleanLiteral { .. } => Ok("bool".to_string()),

            Expr::Identifier { name, token } => self
                .variables
                .get(name)
                .map(|info| info.name.clone())
                .ok_or_else(|| {
                    format!(
                        "Undefined variable '{}' at {}:{}",
                        name, token.line, token.column
                    )
                }),

            Expr::BinaryOp { left, op, right } => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;

                match op.kind {
                    TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Slash => {
                        self.check_numeric_types(&left_type, &right_type, op)
                    }

                    TokenType::EqualEqual
                    | TokenType::NotEqual
                    | TokenType::LessThan
                    | TokenType::LessEqual
                    | TokenType::GreaterThan
                    | TokenType::GreaterEqual => Ok("bool".to_string()),

                    TokenType::And | TokenType::Or => {
                        self.check_bool_types(&left_type, &right_type, op)
                    }

                    TokenType::Equal => Ok("void".to_string()),

                    _ => Err(format!("Unknown operator: {:?}", op.kind)),
                }
            }

            Expr::UnaryOp { op, operand } => {
                let operand_type = self.check_expression(operand)?;

                match op.kind {
                    TokenType::Minus => {
                        if !self.is_numeric_type(&operand_type) {
                            return Err(format!(
                                "Unary operator '{:?}' requires numeric type, got '{}'",
                                op.kind, operand_type
                            ));
                        }
                        Ok(operand_type)
                    }
                    TokenType::Not => {
                        if operand_type != "bool" {
                            return Err(format!(
                                "Unary operator '{:?}' requires bool type, got '{}'",
                                op.kind, operand_type
                            ));
                        }
                        Ok("bool".to_string())
                    }
                    _ => Err(format!("Unknown unary operator: {:?}", op.kind)),
                }
            }

            Expr::Call {
                callee,
                args,
                token,
            } => {
                let name = match **callee {
                    Expr::Identifier { ref name, .. } => name,
                    _ => return Err("Can only call named functions".to_string()),
                };

                if name == "println" || name == "print" {
                    if args.len() != 1 {
                        return Err(format!(
                            "Function '{}' expects 1 argument, got {} at {}:{}",
                            name,
                            args.len(),
                            token.line,
                            token.column
                        ));
                    }
                    let arg_type = self.check_expression(&args[0])?;
                    if arg_type != "str" && !self.is_numeric_type(&arg_type) && arg_type != "bool" && arg_type != "char" {
                        return Err(format!(
                            "Function '{}' expects str or numeric type, got '{}' at {}:{}",
                            name, arg_type, token.line, token.column
                        ));
                    }
                    return Ok("void".to_string());
                }

                if let Some((params, return_type)) = self.functions.get(name) {
                    if args.len() != params.len() {
                        return Err(format!(
                            "Function '{}' expects {} arguments, got {} at {}:{}",
                            name,
                            params.len(),
                            args.len(),
                            token.line,
                            token.column
                        ));
                    }

                    for (arg, (param_name, param_type)) in args.iter().zip(params.iter()) {
                        let arg_type = self.check_expression(arg)?;
                        if arg_type != *param_type {
                            return Err(format!(
                                "Type mismatch for argument '{}' of function '{}': expected '{}' but got '{}' at {}:{}",
                                param_name, name, param_type, arg_type, token.line, token.column
                            ));
                        }
                    }

                    Ok(return_type.clone())
                } else {
                    Err(format!(
                        "Undefined function '{}' at {}:{}",
                        name, token.line, token.column
                    ))
                }
            }

            Expr::OwnershipTransfer { expr, token: _ } => {
                let inner_type = self.check_expression(expr)?;
                Ok(inner_type)
            }
        }
    }

    fn check_numeric_types(
        &self,
        left: &str,
        right: &str,
        op: &crate::token::Token,
    ) -> Result<String, String> {
        if !self.is_numeric_type(left) || !self.is_numeric_type(right) {
            return Err(format!(
                "Operator '{:?}' requires numeric types at {}:{}, got '{}' and '{}'",
                op.kind, op.line, op.column, left, right
            ));
        }

        // Numeric promotion rules
        if left == right {
            return Ok(left.to_string());
        }

        if left == "f64" || right == "f64" {
            Ok("f64".to_string())
        } else if left == "f32" || right == "f32" {
            Ok("f32".to_string())
        } else if left == "i64" || right == "i64" {
            Ok("i64".to_string())
        } else if left == "i32" || right == "i32" {
            Ok("i32".to_string())
        } else if left == "i16" || right == "i16" {
            Ok("i16".to_string())
        } else if left == "i8" || right == "i8" {
            Ok("i8".to_string())
        } else if left == "u64" || right == "u64" {
            Ok("u64".to_string())
        } else if left == "u32" || right == "u32" {
            Ok("u32".to_string())
        } else if left == "u16" || right == "u16" {
            Ok("u16".to_string())
        } else {
            Ok("u8".to_string())
        }
    }

    fn check_bool_types(
        &self,
        left: &str,
        right: &str,
        op: &crate::token::Token,
    ) -> Result<String, String> {
        if left != "bool" || right != "bool" {
            return Err(format!(
                "Operator '{:?}' requires bool types at {}:{}, got '{}' and '{}'",
                op.kind, op.line, op.column, left, right
            ));
        }
        Ok("bool".to_string())
    }

    fn is_numeric_type(&self, t: &str) -> bool {
        matches!(
            t,
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64"
        )
    }

    fn is_valid_type(&self, t: &str) -> bool {
        matches!(
            t,
            "i8" | "i16"
                | "i32"
                | "i64"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "f32"
                | "f64"
                | "bool"
                | "str"
                | "char"
                | "void"
        )
    }
}

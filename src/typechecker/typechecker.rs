use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct TypeInfo {
    name: String,
    is_mutable: bool,
    scope_level: usize,
    is_initialized: bool,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    params: Vec<(String, String)>,
    return_type: String,
    is_defined: bool,
    call_count: usize,
}

pub struct TypeChecker {
    variables: HashMap<String, TypeInfo>,
    functions: HashMap<String, FunctionInfo>,
    errors: Vec<String>,
    warnings: Vec<String>,
    scope_level: usize,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut checker = TypeChecker {
            variables: HashMap::new(),
            functions: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            scope_level: 0,
        };
        
        // Initialize built-in functions
        checker.functions.insert("println".to_string(), FunctionInfo {
            params: vec![("value".to_string(), "any".to_string())],
            return_type: "void".to_string(),
            is_defined: true,
            call_count: 0,
        });
        
        checker
    }

    pub fn check(&mut self, program: &crate::ast::program::Program) -> Result<(), String> {
        // First pass: collect all function signatures
        for stmt in &program.statements {
            if let Stmt::FunctionDecl { name, params, return_type, .. } = stmt {
                self.register_function(name, params, return_type)?;
            }
        }
        
        // Second pass: type check all statements
        for stmt in &program.statements {
            if let Err(e) = self.check_statement(stmt) {
                self.errors.push(e);
            }
        }
        
        // Report results
        if !self.warnings.is_empty() {
            for warning in &self.warnings {
                eprintln!("Warning: {}", warning);
            }
        }
        
        if !self.errors.is_empty() {
            let error_summary = format!("Type checking failed with {} errors:\n{}", 
                                      self.errors.len(), 
                                      self.errors.join("\n"));
            return Err(error_summary);
        }
        
        Ok(())
    }

    fn register_function(&mut self, name: &str, params: &[(String, String)], return_type: &str) -> Result<(), String> {
        if self.functions.contains_key(name) && name != "println" {
            return Err(format!("Function '{}' is already defined", name));
        }
        
        // Validate parameter types
        for (_param_name, param_type) in params {
            if !self.is_valid_type(param_type) {
                return Err(format!("Invalid parameter type '{}' in function '{}'", param_type, name));
            }
        }
        
        if !self.is_valid_type(return_type) {
            return Err(format!("Invalid return type '{}' in function '{}'", return_type, name));
        }
        
        self.functions.insert(name.to_string(), FunctionInfo {
            params: params.to_vec(),
            return_type: return_type.to_string(),
            is_defined: true,
            call_count: 0,
        });
        
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::VariableDecl { name, type_annotation, initializer, is_mutable, token } => {
                let var_type = if let Some(t) = type_annotation {
                    if !self.is_valid_type(t) {
                        return Err(format!("Invalid type '{}' at line {}:{}", t, token.line, token.column));
                    }
                    t.clone()
                } else if let Some(init) = initializer {
                    self.infer_expression_type(init)?
                } else {
                    return Err(format!("Variable '{}' must have type annotation or initializer", name));
                };

                self.variables.insert(name.clone(), TypeInfo {
                    name: var_type,
                    is_mutable: *is_mutable,
                    scope_level: self.scope_level,
                    is_initialized: initializer.is_some(),
                });
            }
            
            Stmt::If { condition, then_branch, else_if_branches, else_branch, .. } => {
                // Check main condition
                let condition_type = self.infer_expression_type(condition)?;
                if condition_type != "bool" {
                    return Err(format!("If condition must be boolean, got '{}'", condition_type));
                }
                
                // Check then branch
                self.scope_level += 1;
                for stmt in then_branch {
                    self.check_statement(stmt)?;
                }
                self.scope_level -= 1;
                
                // Check all else if branches
                for else_if_branch in else_if_branches {
                    let else_if_condition_type = self.infer_expression_type(&else_if_branch.condition)?;
                    if else_if_condition_type != "bool" {
                        return Err(format!("Else if condition must be boolean, got '{}'", else_if_condition_type));
                    }
                    
                    self.scope_level += 1;
                    for stmt in &else_if_branch.body {
                        self.check_statement(stmt)?;
                    }
                    self.scope_level -= 1;
                }
                
                // Check else branch if present
                if let Some(else_stmts) = else_branch {
                    self.scope_level += 1;
                    for stmt in else_stmts {
                        self.check_statement(stmt)?;
                    }
                    self.scope_level -= 1;
                }
            }
            
            Stmt::FunctionDecl { name: _, params, body, .. } => {
                // Enter function scope
                self.scope_level += 1;
                
                // Add parameters to scope
                for (param_name, param_type) in params {
                    self.variables.insert(param_name.clone(), TypeInfo {
                        name: param_type.clone(),
                        is_mutable: false,
                        scope_level: self.scope_level,
                        is_initialized: true,
                    });
                }
                
                // Check function body
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                
                // Exit function scope
                self.variables.retain(|_, info| info.scope_level < self.scope_level);
                self.scope_level -= 1;
            }
            
            _ => {
                // Basic validation for other statements
            }
        }
        Ok(())
    }

    fn infer_expression_type(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::IntegerLiteral { .. } => Ok("i32".to_string()),
            Expr::FloatLiteral { .. } => Ok("f64".to_string()),
            Expr::BooleanLiteral { .. } => Ok("bool".to_string()),
            Expr::CharLiteral { .. } => Ok("char".to_string()),
            Expr::StringLiteral { .. } => Ok("str".to_string()),
            Expr::Identifier { name, .. } => {
                if let Some(var_info) = self.variables.get(name) {
                    Ok(var_info.name.clone())
                } else {
                    Err(format!("Undefined variable '{}'", name))
                }
            }
            Expr::BinaryOp { op, left, right } => {
                let left_type = self.infer_expression_type(left)?;
                let right_type = self.infer_expression_type(right)?;
                
                match op.kind {
                    // Comparison operators return bool
                    crate::token::TokenType::EqualEqual |
                    crate::token::TokenType::NotEqual |
                    crate::token::TokenType::LessThan |
                    crate::token::TokenType::LessEqual |
                    crate::token::TokenType::GreaterThan |
                    crate::token::TokenType::GreaterEqual => Ok("bool".to_string()),
                    
                    // Logical operators return bool
                    crate::token::TokenType::And |
                    crate::token::TokenType::Or => Ok("bool".to_string()),
                    
                    // Arithmetic operators return the operand type (simplified)
                    crate::token::TokenType::Plus |
                    crate::token::TokenType::Minus |
                    crate::token::TokenType::Star |
                    crate::token::TokenType::Slash |
                    crate::token::TokenType::Percent => {
                        if left_type == right_type {
                            Ok(left_type)
                        } else {
                            Ok("i32".to_string()) // Default to i32
                        }
                    }
                    
                    _ => Ok("unknown".to_string()),
                }
            }
            Expr::UnaryOp { op, .. } => {
                match op.kind {
                    crate::token::TokenType::Bang => Ok("bool".to_string()),
                    _ => Ok("i32".to_string()),
                }
            }
            Expr::Call { .. } => Ok("i32".to_string()), // Simplified for now
            _ => Ok("unknown".to_string()),
        }
    }

    fn is_valid_type(&self, t: &str) -> bool {
        matches!(
            t,
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | 
            "f32" | "f64" | "bool" | "str" | "char" | "void" | "any"
        )
    }
}

use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use std::collections::HashMap;

mod tests;

#[derive(Debug, Clone, PartialEq)]
pub enum BorrowType {
    Immutable,
    Mutable,
}

#[derive(Debug, Clone)]
pub struct BorrowInfo {
    pub borrow_type: BorrowType,
    pub scope_level: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct OwnershipInfo {
    pub owner: String,
    pub is_moved: bool,
    pub move_location: Option<(usize, usize)>,
    pub borrows: Vec<BorrowInfo>,
    pub scope_level: usize,
    pub is_mutable: bool,
}

pub struct OwnershipChecker {
    variables: HashMap<String, OwnershipInfo>,
    scope_level: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl OwnershipChecker {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            scope_level: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn check(&mut self, program: &crate::ast::program::Program) -> Result<(), String> {
        for stmt in &program.statements {
            self.check_statement(stmt)?;
        }

        // Clean up borrows that go out of scope
        self.cleanup_scope();

        if !self.warnings.is_empty() {
            for warning in &self.warnings {
                eprintln!("Warning: {}", warning);
            }
        }

        if !self.errors.is_empty() {
            return Err(format!("Ownership errors:\n{}", self.errors.join("\n")));
        }

        Ok(())
    }

    fn check_statement(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::VariableDecl { name, initializer, is_mutable, token: _, .. } => {
                if let Some(init) = initializer {
                    self.check_expression(init)?;
                }
                
                self.variables.insert(name.clone(), OwnershipInfo {
                    owner: name.clone(),
                    is_moved: false,
                    move_location: None,
                    borrows: Vec::new(),
                    scope_level: self.scope_level,
                    is_mutable: *is_mutable,
                });
            }
            
            Stmt::Assignment { target, value, token } => {
                self.check_expression(value)?;
                
                if let Expr::Identifier { name, .. } = target {
                    if let Some(info) = self.variables.get(name) {
                        if info.is_moved {
                            self.errors.push(format!(
                                "Cannot assign to moved variable '{}' at {}:{}", 
                                name, token.line, token.column
                            ));
                        }
                        
                        if !info.borrows.is_empty() {
                            self.errors.push(format!(
                                "Cannot assign to borrowed variable '{}' at {}:{}", 
                                name, token.line, token.column
                            ));
                        }
                    }
                }
            }

            Stmt::FunctionDecl { body, .. } => {
                self.enter_scope();
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
            }

            Stmt::If { condition, then_branch, else_if_branches, else_branch, .. } => {
                self.check_expression(condition)?;
                
                self.enter_scope();
                for stmt in then_branch {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();

                for branch in else_if_branches {
                    self.check_expression(&branch.condition)?;
                    self.enter_scope();
                    for stmt in &branch.body {
                        self.check_statement(stmt)?;
                    }
                    self.exit_scope();
                }

                if let Some(else_stmts) = else_branch {
                    self.enter_scope();
                    for stmt in else_stmts {
                        self.check_statement(stmt)?;
                    }
                    self.exit_scope();
                }
            }

            Stmt::While { condition, body, .. } => {
                self.check_expression(condition)?;
                self.enter_scope();
                for stmt in body {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
            }

            Stmt::Block { statements } => {
                self.enter_scope();
                for stmt in statements {
                    self.check_statement(stmt)?;
                }
                self.exit_scope();
            }
            
            _ => {}
        }
        
        Ok(())
    }

    fn check_expression(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::OwnershipTransfer { expr, token } => {
                if let Expr::Identifier { name, .. } = expr.as_ref() {
                    if let Some(info) = self.variables.get_mut(name) {
                        if info.is_moved {
                            self.errors.push(format!(
                                "Cannot move already moved variable '{}' at {}:{}", 
                                name, token.line, token.column
                            ));
                        } else if !info.borrows.is_empty() {
                            self.errors.push(format!(
                                "Cannot move borrowed variable '{}' at {}:{}", 
                                name, token.line, token.column
                            ));
                        } else {
                            info.is_moved = true;
                            info.move_location = Some((token.line, token.column));
                        }
                    }
                }
            }

            Expr::Borrow { expr, is_mutable, token } => {
                if let Expr::Identifier { name, .. } = expr.as_ref() {
                    let borrow_type = if *is_mutable { BorrowType::Mutable } else { BorrowType::Immutable };
                    self.add_borrow(name, borrow_type, token.line, token.column)?;
                }
                self.check_expression(expr)?;
            }
            
            Expr::BinaryOp { left, right, .. } => {
                self.check_expression(left)?;
                self.check_expression(right)?;
            }

            Expr::Call { callee, args, .. } => {
                self.check_expression(callee)?;
                for arg in args {
                    self.check_expression(arg)?;
                }
            }

            Expr::Identifier { name, token } => {
                if let Some(info) = self.variables.get(name) {
                    if info.is_moved {
                        if let Some((move_line, move_col)) = info.move_location {
                            self.errors.push(format!(
                                "Use of moved variable '{}' at {}:{} (moved at {}:{})", 
                                name, token.line, token.column, move_line, move_col
                            ));
                        }
                    }
                }
            }
            
            _ => {}
        }
        
        Ok(())
    }

    fn add_borrow(&mut self, var_name: &str, borrow_type: BorrowType, line: usize, column: usize) -> Result<(), String> {
        if let Some(info) = self.variables.get_mut(var_name) {
            if info.is_moved {
                return Err(format!(
                    "Cannot borrow moved variable '{}' at {}:{}", 
                    var_name, line, column
                ));
            }

            // Check borrow rules
            match borrow_type {
                BorrowType::Mutable => {
                    if !info.borrows.is_empty() {
                        return Err(format!(
                            "Cannot create mutable borrow of '{}' at {}:{} - already borrowed", 
                            var_name, line, column
                        ));
                    }
                    if !info.is_mutable {
                        return Err(format!(
                            "Cannot create mutable borrow of immutable variable '{}' at {}:{}", 
                            var_name, line, column
                        ));
                    }
                }
                BorrowType::Immutable => {
                    // Check for existing mutable borrows
                    if info.borrows.iter().any(|b| b.borrow_type == BorrowType::Mutable) {
                        return Err(format!(
                            "Cannot create immutable borrow of '{}' at {}:{} - mutably borrowed", 
                            var_name, line, column
                        ));
                    }
                }
            }

            info.borrows.push(BorrowInfo {
                borrow_type,
                scope_level: self.scope_level,
                line,
                column,
            });
        }

        Ok(())
    }

    fn enter_scope(&mut self) {
        self.scope_level += 1;
    }

    fn exit_scope(&mut self) {
        // Remove borrows that go out of scope
        for info in self.variables.values_mut() {
            info.borrows.retain(|borrow| borrow.scope_level < self.scope_level);
        }
        
        // Remove variables that go out of scope
        self.variables.retain(|_, info| info.scope_level < self.scope_level);
        
        self.scope_level -= 1;
    }

    fn cleanup_scope(&mut self) {
        // Final cleanup
        for info in self.variables.values_mut() {
            info.borrows.clear();
        }
    }
}

impl Default for OwnershipChecker {
    fn default() -> Self {
        Self::new()
    }
}

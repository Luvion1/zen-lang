use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;

pub struct StringGenerator {
    strings: Vec<String>,
}

impl StringGenerator {
    pub fn new() -> Self {
        StringGenerator {
            strings: Vec::new(),
        }
    }

    pub fn generate_strings(&mut self, stmt: &Stmt) {
        self.collect_strings(stmt);
    }

    fn collect_strings(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::ExprStmt { expr } => self.collect_strings_from_expr(expr),
            Stmt::VariableDecl { initializer, .. } => {
                if let Some(init) = initializer {
                    self.collect_strings_from_expr(init);
                }
            }
            Stmt::Assignment { value, .. } => {
                self.collect_strings_from_expr(value);
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.collect_strings_from_expr(v);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.collect_strings_from_expr(condition);
                for s in then_branch {
                    self.collect_strings(s);
                }
                if let Some(else_stmts) = else_branch {
                    for s in else_stmts {
                        self.collect_strings(s);
                    }
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.collect_strings_from_expr(condition);
                for s in body {
                    self.collect_strings(s);
                }
            }
            Stmt::For {
                init,
                condition,
                increment,
                body,
                ..
            } => {
                if let Some(init_stmt) = init {
                    self.collect_strings(init_stmt);
                }
                if let Some(cond) = condition {
                    self.collect_strings_from_expr(cond);
                }
                if let Some(inc) = increment {
                    self.collect_strings_from_expr(inc);
                }
                for s in body {
                    self.collect_strings(s);
                }
            }
            Stmt::Block { statements } => {
                for s in statements {
                    self.collect_strings(s);
                }
            }
            Stmt::Match {
                value,
                arms,
                default,
                ..
            } => {
                self.collect_strings_from_expr(value);
                for (pattern, body) in arms {
                    self.collect_strings_from_expr(pattern);
                    for s in body {
                        self.collect_strings(s);
                    }
                }
                if let Some(default_body) = default {
                    for s in default_body {
                        self.collect_strings(s);
                    }
                }
            }

            Stmt::FunctionDecl { body, .. } => {
                for s in body {
                    self.collect_strings(s);
                }
            }
        }
    }

    fn collect_strings_from_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::StringLiteral { value, .. } => {
                self.add_string(value);
            }
            Expr::BinaryOp { left, right, .. } => {
                self.collect_strings_from_expr(left);
                self.collect_strings_from_expr(right);
            }
            Expr::UnaryOp { operand, .. } => {
                self.collect_strings_from_expr(operand);
            }
            Expr::Call { args, .. } => {
                for arg in args {
                    self.collect_strings_from_expr(arg);
                }
            }
            Expr::OwnershipTransfer { expr, .. } => {
                self.collect_strings_from_expr(expr);
            }
            _ => {}
        }
    }

    pub fn add_string(&mut self, value: &str) {
        if !self.strings.iter().any(|s| s == value) {
            self.strings.push(value.to_string());
        }
    }

    pub fn add_string_literal(&mut self, value: &str) -> usize {
        let idx = self.strings.len();
        self.strings.push(value.to_string());
        idx
    }

    pub fn get_string_literal(&self, value: &str) -> Result<(String, usize), String> {
        if let Some(i) = self.strings.iter().position(|s| s == value) {
            return Ok((format!("\"{}\\00\"", escape_for_llvm(value)), i));
        }

        Err(format!(
            "String '{}' was not pre-collected. Call generate_strings first.",
            value
        ))
    }

    pub fn finish(&self) -> &[String] {
        &self.strings
    }
}

impl Default for StringGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Escape special characters for LLVM IR string literals
fn escape_for_llvm(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\n' => result.push_str("\\0A"), // Newline as hex escape
            '\r' => result.push_str("\\0D"), // Carriage return as hex escape
            '\t' => result.push_str("\\09"), // Tab as hex escape
            '"' => result.push_str("\\22"),  // Double quote as hex escape
            '\\' => result.push_str("\\5C"), // Backslash as hex escape
            '%' => result.push_str("\\25"),  // Percent as hex escape
            _ if c.is_ascii_control() => {
                result.push_str(&format!("\\{:02X}", c as u8));
            }
            _ => result.push(c),
        }
    }
    result
}

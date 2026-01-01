use crate::ast::expr::Expr;
use crate::ast::stmt::Stmt;
use crate::codegen::ir::StringGenerator;
use crate::token::TokenType;
use std::collections::HashMap;

#[derive(Default)]
pub struct CodeGenerator {
    functions: HashMap<String, (Vec<String>, String)>,
    variables: HashMap<String, (String, bool, usize)>,
    current_function: Option<String>,
    counter: usize,
    label_counter: usize,
    string_gen: StringGenerator,
    last_register: Option<usize>,
}

const VOID_TYPE: &str = "void";
const I32_TYPE: &str = "i32";

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            variables: HashMap::new(),
            current_function: None,
            counter: 0,
            label_counter: 0,
            string_gen: StringGenerator::new(),
            last_register: None,
        }
    }

    pub fn generate(&mut self, program: &crate::ast::program::Program) -> String {
        let mut ir = String::new();

        ir.push_str("declare i32 @puts(i8*)\n");
        ir.push_str("declare i32 @printf(i8*, ...)\n");
        ir.push_str("declare i32 @sprintf(i8*, i8*, ...)\n");
        ir.push_str("@int_fmt = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\"\n");
        ir.push_str("@int_fmt_no_nl = private unnamed_addr constant [3 x i8] c\"%d\\00\"\n");
        ir.push_str("@float_fmt = private unnamed_addr constant [4 x i8] c\"%f\\0A\\00\"\n");
        ir.push_str("@float_fmt_no_nl = private unnamed_addr constant [3 x i8] c\"%f\\00\"\n\n");

        for stmt in &program.statements {
            self.register_functions(stmt);
        }

        for stmt in &program.statements {
            self.string_gen.generate_strings(stmt);
        }

        let strings = self.string_gen.finish();

        for (i, s) in strings.iter().enumerate() {
            use std::fmt::Write;
            writeln!(ir, "@.str.{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"",
                i, s.len() + 1, self.escape_for_llvm(s)).unwrap();
        }
        #[allow(clippy::single_char_add_str)]
        ir.push_str("\n");

        for stmt in &program.statements {
            self.generate_statement(stmt, &mut ir);
        }

        ir
    }

    fn escape_for_llvm(&self, s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                '\n' => result.push_str("\\0A"),
                '\r' => result.push_str("\\0D"),
                '\t' => result.push_str("\\09"),
                '"' => result.push_str("\\22"),
                '\\' => result.push_str("\\5C"),
                '%' => result.push_str("\\25"),
                _ if c.is_ascii_control() => {
                    result.push_str(&format!("\\{:02X}", c as u8));
                }
                _ => result.push(c),
            }
        }
        result
    }

    fn register_functions(&mut self, stmt: &Stmt) {
        if let Stmt::FunctionDecl {
            name,
            params,
            return_type,
            ..
        } = stmt
        {
            let param_types: Vec<String> = params.iter().map(|(_, t)| t.clone()).collect();
            self.functions
                .insert(name.to_string(), (param_types, return_type.to_string()));
        }
    }

    fn fresh_id(&mut self) -> usize {
        let id = self.counter;
        self.counter += 1;
        self.last_register = Some(id);
        id
    }

    fn fresh_label(&mut self) -> usize {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }

    fn get_llvm_type(&self, zen_type: &str) -> &'static str {
        match zen_type {
            "i8" => "i8",
            "i16" => "i16",
            I32_TYPE => "i32",
            "i64" => "i64",
            "u8" => "i8",
            "u16" => "i16",
            "u32" => "i32",
            "u64" => "i64",
            "f32" => "float",
            "f64" => "double",
            "bool" => "i1",
            "str" => "i8*",
            "char" => "i8",
            VOID_TYPE => "void",
            _ => {
                eprintln!("Warning: Unknown type '{}', defaulting to i32", zen_type);
                I32_TYPE
            }
        }
    }

    fn infer_expression_type(&self, expr: &Expr) -> String {
        match expr {
            Expr::IntegerLiteral { .. } => "i32".to_string(),
            Expr::FloatLiteral { .. } => "f64".to_string(),
            Expr::BooleanLiteral { .. } => "bool".to_string(),
            Expr::CharLiteral { .. } => "char".to_string(),
            Expr::StringLiteral { .. } => "str".to_string(),
            Expr::Identifier { name, .. } => {
                self.variables.get(name)
                    .map(|(t, _, _)| t.clone())
                    .unwrap_or_else(|| {
                        eprintln!("Warning: Cannot infer type for undefined variable '{}'", name);
                        "i32".to_string()
                    })
            }
            Expr::BinaryOp { left, op, .. } => {
                match op.kind {
                    TokenType::EqualEqual | TokenType::NotEqual |
                    TokenType::LessThan | TokenType::LessEqual |
                    TokenType::GreaterThan | TokenType::GreaterEqual |
                    TokenType::And | TokenType::Or => "bool".to_string(),
                    _ => self.infer_expression_type(left)
                }
            }
            Expr::UnaryOp { operand, .. } => self.infer_expression_type(operand),
            Expr::Call { callee, .. } => {
                if let Expr::Identifier { name, .. } = callee.as_ref() {
                    self.functions.get(name)
                        .map(|(_, ret_type)| ret_type.clone())
                        .unwrap_or_else(|| "i32".to_string())
                } else {
                    "i32".to_string()
                }
            }
            _ => "i32".to_string(),
        }
    }

    fn handle_type_coercion(
        &mut self,
        left_val: String,
        right_val: String,
        left_type: &str,
        right_type: &str,
        target_type: &str,
        ir: &mut String,
    ) -> (String, String, String) {
        if left_type == right_type && left_type == target_type {
            return (left_val, right_val, target_type.to_string());
        }

        let mut final_left = left_val;
        let mut final_right = right_val;
        let mut op_type = target_type.to_string();

        // Handle numeric promotions
        if (left_type == "i32" && right_type == "f64") || (left_type == "f64" && right_type == "i32") {
            op_type = "f64".to_string();
            
            if left_type == "i32" {
                let id = self.fresh_id();
                ir.push_str(&format!("  %{} = sitofp i32 {} to double\n", id, final_left));
                final_left = format!("%{}", id);
            }
            
            if right_type == "i32" {
                let id = self.fresh_id();
                ir.push_str(&format!("  %{} = sitofp i32 {} to double\n", id, final_right));
                final_right = format!("%{}", id);
            }
        }

        // Handle boolean conversions
        if target_type == "bool" && (left_type != "bool" || right_type != "bool") {
            if left_type != "bool" {
                let id = self.fresh_id();
                ir.push_str(&format!("  %{} = icmp ne i32 {}, 0\n", id, final_left));
                final_left = format!("%{}", id);
            }
            
            if right_type != "bool" {
                let id = self.fresh_id();
                ir.push_str(&format!("  %{} = icmp ne i32 {}, 0\n", id, final_right));
                final_right = format!("%{}", id);
            }
            op_type = "bool".to_string();
        }

        (final_left, final_right, op_type)
    }

    fn is_compatible_type(&self, target_type: &str, source_type: &str) -> bool {
        // Enhanced type compatibility checking
        match (target_type, source_type) {
            // Exact matches
            (a, b) if a == b => true,
            
            // Numeric promotions
            ("f64", "f32") | ("f64", "i32") | ("f64", "i16") | ("f64", "i8") => true,
            ("f32", "i32") | ("f32", "i16") | ("f32", "i8") => true,
            ("i64", "i32") | ("i64", "i16") | ("i64", "i8") => true,
            ("i32", "i16") | ("i32", "i8") => true,
            ("i16", "i8") => true,
            
            // Unsigned to signed (with warning)
            ("i32", "u32") | ("i16", "u16") | ("i8", "u8") => {
                eprintln!("Warning: Implicit conversion from unsigned to signed type");
                true
            }
            
            // Boolean conversions
            ("bool", "i32") | ("bool", "i16") | ("bool", "i8") => true,
            ("i32", "bool") | ("i16", "bool") | ("i8", "bool") => true,
            
            // Character conversions
            ("char", "i8") | ("i8", "char") => true,
            
            _ => false,
        }
    }

    fn generate_statement(&mut self, stmt: &Stmt, ir: &mut String) {
        #[allow(clippy::single_match)]
        match stmt {
            Stmt::FunctionDecl {
                name,
                params,
                return_type,
                body,
                ..
            } => {
                self.generate_function(name, params, return_type, body, ir);
            }
            _ => {}
        }
    }

    fn generate_function(
        &mut self,
        name: &str,
        params: &[(String, String)],
        return_type: &str,
        body: &[Stmt],
        ir: &mut String,
    ) {
        let old_function = self.current_function.take();
        let old_vars = std::mem::take(&mut self.variables);

        self.current_function = Some(name.to_string());
        self.counter = 0;
        self.label_counter = 0;

        let llvm_return = self.get_llvm_type(return_type);
        ir.push_str(&format!("define {} @{}(", llvm_return, name));

        for (i, (param_name, param_type)) in params.iter().enumerate() {
            if i > 0 {
                ir.push_str(", ");
            }
            let llvm_param_type = self.get_llvm_type(param_type);
            ir.push_str(&format!("{} %{}", llvm_param_type, param_name));
        }

        ir.push_str(") {\n");
        ir.push_str("entry:\n");

        for (param_name, param_type) in params {
            let llvm_param_type = self.get_llvm_type(param_type);
            let id = self.fresh_id();
            ir.push_str(&format!("  %{} = alloca {}\n", id, llvm_param_type));
            ir.push_str(&format!(
                "  store {} %{}, {}* %{}\n",
                llvm_param_type, param_name, llvm_param_type, id
            ));
            self.variables
                .insert(param_name.clone(), (param_type.clone(), false, id));
        }

        let mut last_expr_value: Option<String> = None;
        let mut had_return = false;

        for stmt in body {
            if let Stmt::Return { .. } = stmt {
                had_return = true;
            }
            if let Stmt::ExprStmt { expr } = stmt {
                last_expr_value = Some(self.generate_expression(expr, ir));
                had_return = false;
            } else {
                last_expr_value = None;
                self.generate_function_statement(stmt, ir);
            }
        }

        if !had_return {
            if return_type == VOID_TYPE {
                ir.push_str("  ret void\n");
            } else if let Some(value) = last_expr_value {
                ir.push_str(&format!("  ret {} {}\n", llvm_return, value));
            } else {
                ir.push_str(&format!("  ret {} 0\n", llvm_return));
            }
        }

        ir.push_str("}\n\n");

        self.current_function = old_function;
        self.variables = old_vars;
    }

    fn generate_function_statement(&mut self, stmt: &Stmt, ir: &mut String) {
        match stmt {
            Stmt::VariableDecl {
                name,
                type_annotation,
                initializer,
                is_mutable,
                ..
            } => {
                let zen_type = if let Some(type_ann) = type_annotation {
                    type_ann.as_str()
                } else if let Some(init) = initializer {
                    // Infer type from initializer
                    match init {
                        crate::ast::expr::Expr::StringLiteral { .. } => "str",
                        crate::ast::expr::Expr::IntegerLiteral { .. } => I32_TYPE,
                        crate::ast::expr::Expr::FloatLiteral { .. } => "f64",
                        crate::ast::expr::Expr::BooleanLiteral { .. } => "bool",
                        crate::ast::expr::Expr::CharLiteral { .. } => "char",
                        _ => I32_TYPE,
                    }
                } else {
                    I32_TYPE
                };
                let llvm_type = self.get_llvm_type(zen_type);

                let id = self.fresh_id();
                // Handle string pointer allocation
                if zen_type == "str" {
                    ir.push_str(&format!("  %{} = alloca i8*\n", id));
                } else {
                    ir.push_str(&format!("  %{} = alloca {}\n", id, llvm_type));
                }
                self.variables
                    .insert(name.clone(), (zen_type.to_string(), *is_mutable, id));

                if let Some(init) = initializer {
                    let init_value = self.generate_expression(init, ir);
                    // Handle string types specially
                    if zen_type == "str" {
                        ir.push_str(&format!("  store i8* {}, i8** %{}\n", init_value, id));
                    } else {
                        ir.push_str(&format!(
                            "  store {} {}, {}* %{}\n",
                            llvm_type, init_value, llvm_type, id
                        ));
                    }
                }
            }

            Stmt::Assignment { target, value, .. } => {
                #[allow(clippy::collapsible_match)]
                if let Expr::Identifier { name, .. } = target {
                    if let Some(var_info) = self.variables.get(name).cloned() {
                        let (zen_type, _, alloc_id) = var_info;
                        let llvm_type = self.get_llvm_type(&zen_type);
                        let value_str = self.generate_expression(value, ir);
                        
                        // Handle string assignment specially
                        if zen_type == "str" {
                            ir.push_str(&format!("  store i8* {}, i8** %{}\n", value_str, alloc_id));
                        } else {
                            ir.push_str(&format!(
                                "  store {} {}, {}* %{}\n",
                                llvm_type, value_str, llvm_type, alloc_id
                            ));
                        }
                    }
                }
            }

            Stmt::Return { value, .. } => {
                let return_type = if let Some(fn_name) = &self.current_function {
                    if let Some((_, ret)) = self.functions.get(fn_name) {
                        self.get_llvm_type(ret)
                    } else {
                        "i32"
                    }
                } else {
                    "i32"
                };

                if let Some(v) = value {
                    let value_str = self.generate_expression(v, ir);
                    ir.push_str(&format!("  ret {} {}\n", return_type, value_str));
                } else {
                    ir.push_str(&format!("  ret {} 0\n", return_type));
                }
            }

            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
                ..
            } => {
                let cond_value = self.generate_expression(condition, ir);
                
                // Convert i32 to i1 for branch condition
                let bool_cond = if self.infer_expression_type(condition) == "bool" {
                    // If it's already a comparison result (i32 from our conversion), convert back to i1
                    let bool_id = self.fresh_id();
                    ir.push_str(&format!("  %{} = icmp ne i32 {}, 0\n", bool_id, cond_value));
                    format!("%{}", bool_id)
                } else {
                    // For other types, convert to bool
                    let bool_id = self.fresh_id();
                    ir.push_str(&format!("  %{} = icmp ne i32 {}, 0\n", bool_id, cond_value));
                    format!("%{}", bool_id)
                };

                let then_label = self.fresh_label();
                let end_label = self.fresh_label();
                
                // Determine the first alternative label
                let first_alt_label = if !else_if_branches.is_empty() {
                    self.fresh_label()
                } else if else_branch.is_some() {
                    self.fresh_label()
                } else {
                    end_label
                };

                // Branch to then or first alternative
                ir.push_str(&format!(
                    "  br i1 {}, label %then.{}, label %{}{}{}",
                    bool_cond, 
                    then_label,
                    if !else_if_branches.is_empty() { "elseif." } else if else_branch.is_some() { "else." } else { "end." },
                    first_alt_label,
                    "\n"
                ));

                // Generate then branch
                ir.push_str(&format!("then.{}:\n", then_label));
                let mut then_terminated = false;
                for stmt in then_branch {
                    if matches!(stmt, Stmt::Return { .. }) {
                        then_terminated = true;
                    }
                    self.generate_function_statement(stmt, ir);
                }
                if !then_terminated {
                    ir.push_str(&format!("  br label %end.{}\n", end_label));
                }

                // Generate else if branches
                let mut current_label = first_alt_label;
                for (i, else_if_branch) in else_if_branches.iter().enumerate() {
                    if !else_if_branches.is_empty() {
                        ir.push_str(&format!("elseif.{}:\n", current_label));
                    }
                    
                    // Generate condition for this else if
                    let else_if_cond_value = self.generate_expression(&else_if_branch.condition, ir);
                    let else_if_bool_cond = {
                        let bool_id = self.fresh_id();
                        ir.push_str(&format!("  %{} = icmp ne i32 {}, 0\n", bool_id, else_if_cond_value));
                        format!("%{}", bool_id)
                    };
                    
                    let else_if_then_label = self.fresh_label();
                    
                    // Determine next alternative label
                    let next_alt_label = if i + 1 < else_if_branches.len() {
                        self.fresh_label()
                    } else if else_branch.is_some() {
                        self.fresh_label()
                    } else {
                        end_label
                    };
                    
                    // Branch for this else if
                    ir.push_str(&format!(
                        "  br i1 {}, label %then.{}, label %{}{}{}",
                        else_if_bool_cond,
                        else_if_then_label,
                        if i + 1 < else_if_branches.len() { "elseif." } else if else_branch.is_some() { "else." } else { "end." },
                        next_alt_label,
                        "\n"
                    ));
                    
                    // Generate else if body
                    ir.push_str(&format!("then.{}:\n", else_if_then_label));
                    let mut else_if_terminated = false;
                    for stmt in &else_if_branch.body {
                        if matches!(stmt, Stmt::Return { .. }) {
                            else_if_terminated = true;
                        }
                        self.generate_function_statement(stmt, ir);
                    }
                    if !else_if_terminated {
                        ir.push_str(&format!("  br label %end.{}\n", end_label));
                    }
                    
                    current_label = next_alt_label;
                }

                // Generate final else branch if present
                if let Some(else_stmts) = else_branch {
                    ir.push_str(&format!("else.{}:\n", current_label));
                    let mut else_terminated = false;
                    for stmt in else_stmts {
                        if matches!(stmt, Stmt::Return { .. }) {
                            else_terminated = true;
                        }
                        self.generate_function_statement(stmt, ir);
                    }
                    if !else_terminated {
                        ir.push_str(&format!("  br label %end.{}\n", end_label));
                    }
                } else if else_if_branches.is_empty() {
                    // No else if branches and no else - current_label is already end_label
                } else {
                    // We have else if branches but no final else
                    ir.push_str(&format!("end.{}:\n", current_label));
                    ir.push_str(&format!("  br label %end.{}\n", end_label));
                }

                ir.push_str(&format!("end.{}:\n", end_label));
            }

            Stmt::While {
                condition, body, ..
            } => {
                let cond_label = self.fresh_label();
                let body_label = self.fresh_label();
                let end_label = self.fresh_label();

                ir.push_str(&format!("  br label %cond.{}\n", cond_label));

                ir.push_str(&format!("cond.{}:\n", cond_label));
                let cond_value = self.generate_expression(condition, ir);
                
                // Convert to i1 for branch condition
                let bool_cond = {
                    let bool_id = self.fresh_id();
                    ir.push_str(&format!("  %{} = icmp ne i32 {}, 0\n", bool_id, cond_value));
                    format!("%{}", bool_id)
                };
                
                ir.push_str(&format!(
                    "  br i1 {}, label %body.{}, label %end.{}\n",
                    bool_cond, body_label, end_label
                ));

                ir.push_str(&format!("body.{}:\n", body_label));
                for stmt in body {
                    self.generate_function_statement(stmt, ir);
                }
                ir.push_str(&format!("  br label %cond.{}\n", cond_label));

                ir.push_str(&format!("end.{}:\n", end_label));
            }

            Stmt::For {
                init,
                condition,
                increment,
                body,
                ..
            } => {
                if let Some(init_stmt) = init {
                    self.generate_function_statement(init_stmt, ir);
                }

                let cond_label = self.fresh_label();
                let body_label = self.fresh_label();
                let _inc_label = self.fresh_label();
                let end_label = self.fresh_label();

                ir.push_str(&format!("  br label %cond.{}\n", cond_label));

                ir.push_str(&format!("cond.{}:\n", cond_label));
                if let Some(cond) = condition {
                    let cond_value = self.generate_expression(cond, ir);
                    
                    // Convert to i1 for branch condition
                    let bool_cond = {
                        let bool_id = self.fresh_id();
                        ir.push_str(&format!("  %{} = icmp ne i32 {}, 0\n", bool_id, cond_value));
                        format!("%{}", bool_id)
                    };
                    
                    ir.push_str(&format!(
                        "  br i1 {}, label %body.{}, label %end.{}\n",
                        bool_cond, body_label, end_label
                    ));
                } else {
                    ir.push_str(&format!("  br label %body.{}\n", body_label));
                }

                ir.push_str(&format!("body.{}:\n", body_label));
                for stmt in body {
                    self.generate_function_statement(stmt, ir);
                }
                if let Some(inc) = increment {
                    // Handle assignment in increment
                    if let Expr::BinaryOp { left, op, right } = inc {
                        if matches!(op.kind, TokenType::Equal) {
                            if let Expr::Identifier { name, .. } = left.as_ref() {
                                if let Some(var_info) = self.variables.get(name).cloned() {
                                    let (zen_type, _, alloc_id) = var_info;
                                    let llvm_type = self.get_llvm_type(&zen_type);
                                    let value_str = self.generate_expression(right, ir);
                                    ir.push_str(&format!(
                                        "  store {} {}, {}* %{}\n",
                                        llvm_type, value_str, llvm_type, alloc_id
                                    ));
                                }
                            }
                        }
                    } else {
                        self.generate_expression(inc, ir);
                    }
                }
                ir.push_str(&format!("  br label %cond.{}\n", cond_label));

                ir.push_str(&format!("end.{}:\n", end_label));
            }

            Stmt::ExprStmt { expr } => {
                self.generate_expression(expr, ir);
            }

            Stmt::Block { statements } => {
                for stmt in statements {
                    self.generate_function_statement(stmt, ir);
                }
            }

            _ => {}
        }
    }

    fn generate_expression(&mut self, expr: &Expr, ir: &mut String) -> String {
        match expr {
            Expr::IntegerLiteral { value, .. } => {
                // Enhanced integer literal handling with validation
                match value.parse::<i64>() {
                    Ok(val) if val >= i32::MIN as i64 && val <= i32::MAX as i64 => {
                        val.to_string()
                    }
                    Ok(val) => {
                        eprintln!("Warning: Integer literal {} may overflow i32, truncating", val);
                        (val as i32).to_string()
                    }
                    Err(_) => {
                        eprintln!("Error: Invalid integer literal {}", value);
                        "0".to_string()
                    }
                }
            }

            Expr::FloatLiteral { value, .. } => {
                // Enhanced float handling with precision control
                if value.is_finite() {
                    if value.fract() == 0.0 {
                        format!("{:.1}", value)
                    } else {
                        format!("{:.6}", value)
                    }
                } else {
                    eprintln!("Warning: Non-finite float value, using 0.0");
                    "0.0".to_string()
                }
            }

            Expr::BooleanLiteral { value, .. } => {
                if *value { "1" } else { "0" }.to_string()
            }

            Expr::CharLiteral { value, .. } => {
                let ascii_value = *value as u8;
                // Validate ASCII range
                if ascii_value <= 127 {
                    ascii_value.to_string()
                } else {
                    eprintln!("Warning: Non-ASCII character, using 0");
                    "0".to_string()
                }
            }

            Expr::StringLiteral { value, .. } => {
                self.generate_string_literal(value, ir)
            }

            Expr::InterpolatedString { parts, .. } => {
                self.generate_interpolated_string(parts, ir)
            }

            Expr::Identifier { name, .. } => {
                // Enhanced identifier resolution with validation
                if let Some(var_info) = self.variables.get(name).cloned() {
                    let (zen_type, _, alloc_id) = var_info;
                    let llvm_type = self.get_llvm_type(&zen_type);
                    let id = self.fresh_id();
                    
                    // Enhanced type-specific loading
                    match zen_type.as_str() {
                        "str" => {
                            ir.push_str(&format!("  %{} = load i8*, i8** %{}\n", id, alloc_id));
                        }
                        "bool" => {
                            ir.push_str(&format!("  %{} = load i1, i1* %{}\n", id, alloc_id));
                        }
                        "char" => {
                            ir.push_str(&format!("  %{} = load i8, i8* %{}\n", id, alloc_id));
                        }
                        _ => {
                            ir.push_str(&format!(
                                "  %{} = load {}, {}* %{}\n",
                                id, llvm_type, llvm_type, alloc_id
                            ));
                        }
                    }
                    format!("%{}", id)
                } else {
                    eprintln!("Error: Undefined variable '{}'", name);
                    format!("%{}", name)
                }
            }

            Expr::BinaryOp { left, op, right } => {
                let left_type = self.infer_expression_type(left);
                let right_type = self.infer_expression_type(right);
                
                let left_val = self.generate_expression(left, ir);
                let right_val = self.generate_expression(right, ir);

                // Handle comparison operations that return bool
                let result = match op.kind {
                    TokenType::EqualEqual | TokenType::NotEqual |
                    TokenType::LessThan | TokenType::LessEqual |
                    TokenType::GreaterThan | TokenType::GreaterEqual => {
                        let op_str = if left_type == "f64" || right_type == "f64" {
                            match op.kind {
                                TokenType::EqualEqual => "fcmp oeq double",
                                TokenType::NotEqual => "fcmp one double",
                                TokenType::LessThan => "fcmp olt double",
                                TokenType::LessEqual => "fcmp ole double",
                                TokenType::GreaterThan => "fcmp ogt double",
                                TokenType::GreaterEqual => "fcmp oge double",
                                _ => "fcmp oeq double",
                            }
                        } else {
                            match op.kind {
                                TokenType::EqualEqual => "icmp eq i32",
                                TokenType::NotEqual => "icmp ne i32",
                                TokenType::LessThan => "icmp slt i32",
                                TokenType::LessEqual => "icmp sle i32",
                                TokenType::GreaterThan => "icmp sgt i32",
                                TokenType::GreaterEqual => "icmp sge i32",
                                _ => "icmp eq i32",
                            }
                        };
                        let id = self.fresh_id();
                        ir.push_str(&format!("  %{} = {} {}, {}\n", id, op_str, left_val, right_val));
                        
                        // Convert i1 result to i32 for compatibility
                        let conv_id = self.fresh_id();
                        ir.push_str(&format!("  %{} = zext i1 %{} to i32\n", conv_id, id));
                        format!("%{}", conv_id)
                    }
                    
                    TokenType::And | TokenType::Or => {
                        // For logical operations, work with i1 directly
                        let left_bool_id = self.fresh_id();
                        let right_bool_id = self.fresh_id();
                        let result_id = self.fresh_id();
                        let final_id = self.fresh_id();
                        
                        // Convert operands to i1
                        ir.push_str(&format!("  %{} = icmp ne i32 {}, 0\n", left_bool_id, left_val));
                        ir.push_str(&format!("  %{} = icmp ne i32 {}, 0\n", right_bool_id, right_val));
                        
                        let op_str = match op.kind {
                            TokenType::And => "and i1",
                            TokenType::Or => "or i1",
                            _ => "and i1",
                        };
                        ir.push_str(&format!("  %{} = {} %{}, %{}\n", result_id, op_str, left_bool_id, right_bool_id));
                        
                        // Convert i1 result to i32 for compatibility
                        ir.push_str(&format!("  %{} = zext i1 %{} to i32\n", final_id, result_id));
                        format!("%{}", final_id)
                    }
                    
                    _ => {
                        // Arithmetic operations
                        let id = self.fresh_id();
                        let op_str = if left_type == "f64" || right_type == "f64" {
                            match op.kind {
                                TokenType::Plus => "fadd double",
                                TokenType::Minus => "fsub double",
                                TokenType::Star => "fmul double",
                                TokenType::Slash => "fdiv double",
                                TokenType::Percent => "frem double",
                                _ => "fadd double",
                            }
                        } else {
                            match op.kind {
                                TokenType::Plus => "add i32",
                                TokenType::Minus => "sub i32",
                                TokenType::Star => "mul i32",
                                TokenType::Slash => "sdiv i32",
                                TokenType::Percent => "srem i32",
                                _ => "add i32",
                            }
                        };
                        ir.push_str(&format!("  %{} = {} {}, {}\n", id, op_str, left_val, right_val));
                        format!("%{}", id)
                    }
                };
                
                result
            }

            Expr::UnaryOp { op, operand } => {
                let operand_val = self.generate_expression(operand, ir);

                match op.kind {
                    TokenType::Minus => {
                        let id = self.fresh_id();
                        ir.push_str(&format!("  %{} = sub i32 0, {}\n", id, operand_val));
                        format!("%{}", id)
                    }
                    TokenType::Not => {
                        // Convert i32 to i1 first, then negate, then back to i32
                        let bool_id = self.fresh_id();
                        let not_id = self.fresh_id();
                        let final_id = self.fresh_id();
                        ir.push_str(&format!("  %{} = icmp ne i32 {}, 0\n", bool_id, operand_val));
                        ir.push_str(&format!("  %{} = xor i1 %{}, true\n", not_id, bool_id));
                        ir.push_str(&format!("  %{} = zext i1 %{} to i32\n", final_id, not_id));
                        format!("%{}", final_id)
                    }
                    _ => {
                        let id = self.fresh_id();
                        ir.push_str(&format!("  %{} = sub i32 0, {}\n", id, operand_val));
                        format!("%{}", id)
                    }
                }
            }

            Expr::Call { callee, args, .. } => {
                if let Expr::Identifier { name, .. } = callee.as_ref() {
                    if name == "println" || name == "print" {
                        for arg in args {
                            match arg {
                                Expr::StringLiteral { .. } => {
                                    let val = self.generate_expression(arg, ir);
                                    let call_id = self.fresh_id();
                                    ir.push_str(&format!(
                                        "  %{} = call i32 @puts(i8* {})\n",
                                        call_id, val
                                    ));
                                }
                                Expr::BooleanLiteral { .. } => {
                                    let val = self.generate_expression(arg, ir);
                                    // Convert i1 to i32 for printing
                                    let conv_id = self.fresh_id();
                                    ir.push_str(&format!("  %{} = zext i1 {} to i32\n", conv_id, val));
                                    let fmt_id = self.fresh_id();
                                    ir.push_str(&format!("  %{} = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @int_fmt, i64 0, i64 0), i32 %{})\n",
                                        fmt_id, conv_id));
                                }
                                Expr::CharLiteral { .. } => {
                                    let val = self.generate_expression(arg, ir);
                                    // Convert i8 to i32 for printing
                                    let conv_id = self.fresh_id();
                                    ir.push_str(&format!("  %{} = zext i8 {} to i32\n", conv_id, val));
                                    let fmt_id = self.fresh_id();
                                    ir.push_str(&format!("  %{} = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @int_fmt, i64 0, i64 0), i32 %{})\n",
                                        fmt_id, conv_id));
                                }
                                Expr::IntegerLiteral { .. } | Expr::FloatLiteral { .. } => {
                                    let val = self.generate_expression(arg, ir);
                                    let (fmt_name, val_type) =
                                        if matches!(arg, Expr::FloatLiteral { .. }) {
                                            ("@float_fmt", "double")
                                        } else {
                                            ("@int_fmt", "i32")
                                        };
                                    let fmt_id = self.fresh_id();
                                    ir.push_str(&format!("  %{} = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* {}, i64 0, i64 0), {} {})\n",
                                        fmt_id, fmt_name, val_type, val));
                                }
                                Expr::Identifier { name, .. } => {
                                    let val = self.generate_expression(arg, ir);
                                    let is_float = self
                                        .variables
                                        .get(name)
                                        .is_some_and(|(t, _, _)| t == "f64" || t == "f32");
                                    let is_bool = self
                                        .variables
                                        .get(name)
                                        .is_some_and(|(t, _, _)| t == "bool");
                                    let is_string = self
                                        .variables
                                        .get(name)
                                        .is_some_and(|(t, _, _)| t == "str");
                                    let is_char = self
                                        .variables
                                        .get(name)
                                        .is_some_and(|(t, _, _)| t == "char");
                                    
                                    if is_string {
                                        let call_id = self.fresh_id();
                                        ir.push_str(&format!(
                                            "  %{} = call i32 @puts(i8* {})\n",
                                            call_id, val
                                        ));
                                    } else if is_char {
                                        // Convert i8 to i32 for printing
                                        let conv_id = self.fresh_id();
                                        ir.push_str(&format!("  %{} = zext i8 {} to i32\n", conv_id, val));
                                        let fmt_id = self.fresh_id();
                                        ir.push_str(&format!("  %{} = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @int_fmt, i64 0, i64 0), i32 %{})\n",
                                            fmt_id, conv_id));
                                    } else {
                                        let (fmt_name, val_type, final_val) = if is_float {
                                            ("@float_fmt", "double", val)
                                        } else if is_bool {
                                            // Convert i1 to i32 for printing
                                            let conv_id = self.fresh_id();
                                            ir.push_str(&format!("  %{} = zext i1 {} to i32\n", conv_id, val));
                                            ("@int_fmt", "i32", format!("%{}", conv_id))
                                        } else {
                                            ("@int_fmt", "i32", val)
                                        };
                                        let fmt_id = self.fresh_id();
                                        ir.push_str(&format!("  %{} = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* {}, i64 0, i64 0), {} {})\n",
                                            fmt_id, fmt_name, val_type, final_val));
                                    }
                                }
                                Expr::BinaryOp { op, .. } => {
                                    let val = self.generate_expression(arg, ir);
                                    let is_float = matches!(arg, Expr::BinaryOp { left, right, .. }
                                        if matches!(left.as_ref(), Expr::FloatLiteral { .. }) || matches!(right.as_ref(), Expr::FloatLiteral { .. }) ||
                                            matches!(left.as_ref(), Expr::Identifier { name, .. } if self.variables.get(name).is_some_and(|(t,_,_)| t=="f64"||t=="f32")) ||
                                            matches!(right.as_ref(), Expr::Identifier { name, .. } if self.variables.get(name).is_some_and(|(t,_,_)| t=="f64"||t=="f32")));
                                    
                                    let is_bool = matches!(op.kind, TokenType::And | TokenType::Or | TokenType::EqualEqual | TokenType::NotEqual | TokenType::LessThan | TokenType::LessEqual | TokenType::GreaterThan | TokenType::GreaterEqual);
                                    
                                    let (fmt_name, val_type, final_val) = if is_float {
                                        ("@float_fmt", "double", val)
                                    } else if is_bool {
                                        // Convert i1 to i32 for printing
                                        let conv_id = self.fresh_id();
                                        ir.push_str(&format!("  %{} = zext i1 {} to i32\n", conv_id, val));
                                        ("@int_fmt", "i32", format!("%{}", conv_id))
                                    } else {
                                        ("@int_fmt", "i32", val)
                                    };
                                    let fmt_id = self.fresh_id();
                                    ir.push_str(&format!("  %{} = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* {}, i64 0, i64 0), {} {})\n",
                                        fmt_id, fmt_name, val_type, final_val));
                                }
                                Expr::Call { .. } => {
                                    let val = self.generate_expression(arg, ir);
                                    // For function calls, assume i32 return type for now
                                    let fmt_id = self.fresh_id();
                                    ir.push_str(&format!("  %{} = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @int_fmt, i64 0, i64 0), i32 {})\n",
                                        fmt_id, val));
                                }
                                _ => {
                                    self.generate_expression(arg, ir);
                                }
                            }
                        }
                        String::new()
                    } else if let Some((params, return_type)) = self.functions.get(name) {
                        let llvm_return = self.get_llvm_type(return_type);
                        let return_type_clone = return_type.clone();
                        let mut arg_values = Vec::new();
                        let params_clone = params.clone();
                        for (arg, param_type) in args.iter().zip(params_clone.iter()) {
                            let llvm_param_type = self.get_llvm_type(param_type);
                            let arg_value = self.generate_expression(arg, ir);
                            arg_values.push(format!("{} {}", llvm_param_type, arg_value));
                        }
                        if return_type_clone == VOID_TYPE {
                            ir.push_str(&format!(
                                "  call void @{}({})\n",
                                name,
                                arg_values.join(", ")
                            ));
                            String::new()
                        } else {
                            let id = self.fresh_id();
                            ir.push_str(&format!(
                                "  %{} = call {} @{}({})\n",
                                id,
                                llvm_return,
                                name,
                                arg_values.join(", ")
                            ));
                            format!("%{}", id)
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            }

            Expr::OwnershipTransfer { expr, .. } => self.generate_expression(expr, ir),
        }
    }

    fn generate_string_literal(&mut self, value: &str, ir: &mut String) -> String {
        let (_, idx) = match self.string_gen.get_string_literal(value) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error: {}", e);
                return "null".to_string();
            }
        };
        let ptr_id = self.fresh_id();
        ir.push_str(&format!(
            "  %{} = getelementptr inbounds [{} x i8], [{} x i8]* @.str.{}, i64 0, i64 0\n",
            ptr_id,
            value.len() + 1,
            value.len() + 1,
            idx
        ));
        format!("%{}", ptr_id)
    }

    fn generate_interpolated_string(&mut self, parts: &[crate::ast::expr::StringPart], ir: &mut String) -> String {
        // Simple approach: print each part separately
        for part in parts {
            match part {
                crate::ast::expr::StringPart::Text(text) => {
                    if !text.is_empty() {
                        let text_literal = Expr::StringLiteral {
                            value: text.clone(),
                            token: crate::token::Token::new(
                                crate::token::TokenType::StringLiteral,
                                format!("\"{}\"", text),
                                1, 1
                            ),
                        };
                        let val = self.generate_expression(&text_literal, ir);
                        let call_id = self.fresh_id();
                        ir.push_str(&format!(
                            "  %{} = call i32 @printf(i8* {})\n",
                            call_id, val
                        ));
                    }
                }
                crate::ast::expr::StringPart::Variable(var_name) => {
                    if let Some((var_type, _, alloc_id)) = self.variables.get(var_name).cloned() {
                        match var_type.as_str() {
                            "i32" => {
                                let load_id = self.fresh_id();
                                ir.push_str(&format!("  %{} = load i32, i32* %{}\n", load_id, alloc_id));
                                let fmt_id = self.fresh_id();
                                ir.push_str(&format!("  %{} = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @int_fmt_no_nl, i64 0, i64 0), i32 %{})\n",
                                    fmt_id, load_id));
                            }
                            "str" => {
                                let load_id = self.fresh_id();
                                ir.push_str(&format!("  %{} = load i8*, i8** %{}\n", load_id, alloc_id));
                                let call_id = self.fresh_id();
                                ir.push_str(&format!(
                                    "  %{} = call i32 @printf(i8* %{})\n",
                                    call_id, load_id
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                crate::ast::expr::StringPart::Expression(expr_str) => {
                    // For now, handle simple function calls like add(result, result)
                    // This is a simplified implementation - in a full compiler, 
                    // we'd parse and evaluate the expression properly
                    if expr_str.starts_with("add(") && expr_str.ends_with(')') {
                        // Extract arguments - very basic parsing
                        let args_str = &expr_str[4..expr_str.len()-1];
                        let args: Vec<&str> = args_str.split(", ").collect();
                        
                        if args.len() == 2 {
                            // Load both arguments
                            let mut arg_values = Vec::new();
                            for arg in args {
                                if let Some((_, _, alloc_id)) = self.variables.get(arg.trim()).cloned() {
                                    let load_id = self.fresh_id();
                                    ir.push_str(&format!("  %{} = load i32, i32* %{}\n", load_id, alloc_id));
                                    arg_values.push(format!("i32 %{}", load_id));
                                }
                            }
                            
                            if arg_values.len() == 2 {
                                // Call the function
                                let call_id = self.fresh_id();
                                ir.push_str(&format!(
                                    "  %{} = call i32 @add({})\n",
                                    call_id, arg_values.join(", ")
                                ));
                                
                                // Print the result
                                let fmt_id = self.fresh_id();
                                ir.push_str(&format!("  %{} = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([3 x i8], [3 x i8]* @int_fmt_no_nl, i64 0, i64 0), i32 %{})\n",
                                    fmt_id, call_id));
                            }
                        }
                    }
                }
            }
        }
        
        // Return empty string since we're printing directly
        String::new()
    }
}

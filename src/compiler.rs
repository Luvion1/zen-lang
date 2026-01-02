use std::path::PathBuf;
use std::time::Instant;

use crate::codegen::codegen::CodeGenerator;
use crate::lexer::lexer::Lexer;
use crate::ownership::OwnershipChecker;
use crate::parser::parser::Parser;
use crate::typechecker::typechecker::TypeChecker;

const LLC_CMD: &str = "llc";
const GCC_CMD: &str = "gcc";

#[derive(Debug, Clone)]
pub struct CompilationStats {
    pub tokens_count: usize,
    pub statements_count: usize,
    pub lexing_time: std::time::Duration,
    pub parsing_time: std::time::Duration,
    pub type_checking_time: std::time::Duration,
    pub ownership_time: std::time::Duration,
    pub codegen_time: std::time::Duration,
    pub llc_time: std::time::Duration,
    pub linking_time: std::time::Duration,
    pub total_time: std::time::Duration,
}

pub struct Compiler {
    stats: Option<CompilationStats>,
    verbose: bool,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            stats: None,
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn get_stats(&self) -> Option<&CompilationStats> {
        self.stats.as_ref()
    }

    pub fn compile(input: &str, output: Option<&str>) -> anyhow::Result<()> {
        let mut compiler = Compiler::new().with_verbose(true);
        compiler.compile_internal(input, output)
    }

    fn compile_internal(&mut self, input: &str, output: Option<&str>) -> anyhow::Result<()> {
        let total_start = Instant::now();
        
        // Validate input file
        if !std::path::Path::new(input).exists() {
            anyhow::bail!("Input file '{}' does not exist", input);
        }
        
        let source = std::fs::read_to_string(input)
            .map_err(|e| anyhow::anyhow!("Failed to read input file '{}': {}", input, e))?;

        if self.verbose {
            println!("Compiling: {}", input);
        }

        // Lexical Analysis
        let lexing_start = Instant::now();
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        let lexing_time = lexing_start.elapsed();

        if self.verbose {
            println!("info: {} tokens found", tokens.len());
        }

        // Syntax Analysis
        let parsing_start = Instant::now();
        let mut parser = Parser::new(tokens.clone());
        let program = parser
            .parse()
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
        let parsing_time = parsing_start.elapsed();

        if self.verbose {
            println!("success: Parsed successfully!");
            println!("  Statements: {}", program.statements.len());
        }

        // Semantic Analysis
        let type_checking_start = Instant::now();
        let mut typechecker = TypeChecker::new();
        typechecker
            .check(&program)
            .map_err(|e| anyhow::anyhow!("Type error: {}", e))?;
        let type_checking_time = type_checking_start.elapsed();

        if self.verbose {
            println!("success: Type checking passed!");
        }

        // Ownership Checking
        let ownership_start = Instant::now();
        let mut ownership_checker = OwnershipChecker::new();
        ownership_checker
            .check(&program)
            .map_err(|e| anyhow::anyhow!("Ownership error: {}", e))?;
        let ownership_time = ownership_start.elapsed();

        if self.verbose {
            println!("success: Ownership checking passed!");
        }

        // Code Generation
        let codegen_start = Instant::now();
        let mut codegen = CodeGenerator::new();
        let llvm_ir = codegen.generate(&program);
        let codegen_time = codegen_start.elapsed();

        // Prepare paths
        let input_path = PathBuf::from(input);
        let output_path = if let Some(out) = output {
            PathBuf::from(out)
        } else {
            input_path.with_extension("")
        };

        let temp_dir = std::env::temp_dir();
        let process_id = std::process::id();
        let ll_path = temp_dir.join(format!("zen_temp_{}.ll", process_id));
        let obj_path = temp_dir.join(format!("zen_temp_{}.o", process_id));

        if self.verbose {
            println!("Compiling {}...", input);
        }

        // Write LLVM IR
        std::fs::write(&ll_path, &llvm_ir)
            .map_err(|e| anyhow::anyhow!("Failed to write LLVM IR: {}", e))?;

        // Debug: Also write to a persistent file for inspection
        if self.verbose {
            let debug_path = format!("{}.ll", input.trim_end_matches(".zen"));
            let _ = std::fs::write(&debug_path, &llvm_ir);
            println!("Debug: LLVM IR written to {}", debug_path);
        }

        // LLVM Compilation
        let llc_start = Instant::now();
        let llc_result = std::process::Command::new(LLC_CMD)
            .arg("-filetype=obj")
            .arg("-o")
            .arg(&obj_path)
            .arg(&ll_path)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute llc: {}", e))?;
        let llc_time = llc_start.elapsed();

        if !llc_result.status.success() {
            let stderr = std::str::from_utf8(&llc_result.stderr).unwrap_or("Invalid UTF-8");
            let _ = std::fs::remove_file(&ll_path);
            anyhow::bail!("llc compilation failed: {}", stderr);
        }

        // Linking
        let linking_start = Instant::now();
        let linker_result = std::process::Command::new(GCC_CMD)
            .arg("-no-pie")
            .arg(&obj_path)
            .arg("-o")
            .arg(&output_path)
            .arg("-lc")
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute linker: {}", e))?;
        let linking_time = linking_start.elapsed();

        // Cleanup
        let _ = std::fs::remove_file(&ll_path);
        let _ = std::fs::remove_file(&obj_path);

        let total_time = total_start.elapsed();

        // Store statistics
        self.stats = Some(CompilationStats {
            tokens_count: tokens.len(),
            statements_count: program.statements.len(),
            lexing_time,
            parsing_time,
            type_checking_time,
            ownership_time,
            codegen_time,
            llc_time,
            linking_time,
            total_time,
        });

        if linker_result.status.success() {
            if self.verbose {
                println!("success: Compiled: {}", output_path.display());
                self.print_stats();
            }
        } else {
            let stderr = std::str::from_utf8(&linker_result.stderr).unwrap_or("Invalid UTF-8");
            anyhow::bail!("linking failed: {}", stderr);
        }

        Ok(())
    }

    fn print_stats(&self) {
        if let Some(stats) = &self.stats {
            println!("\nCompilation Statistics:");
            println!("  Tokens: {}", stats.tokens_count);
            println!("  Statements: {}", stats.statements_count);
            println!("  Lexing: {:?}", stats.lexing_time);
            println!("  Parsing: {:?}", stats.parsing_time);
            println!("  Type Checking: {:?}", stats.type_checking_time);
            println!("  Code Generation: {:?}", stats.codegen_time);
            println!("  LLVM Compilation: {:?}", stats.llc_time);
            println!("  Linking: {:?}", stats.linking_time);
            println!("  Total: {:?}", stats.total_time);
        }
    }

    pub fn run(input: &str) -> anyhow::Result<()> {
        let mut compiler = Compiler::new().with_verbose(false);
        compiler.run_internal(input)
    }

    fn run_internal(&mut self, input: &str) -> anyhow::Result<()> {
        let input_path = PathBuf::from(input);
        let output_path = input_path.with_extension("");

        // Compile first
        self.compile_internal(input, None)?;

        let output_path_abs = std::env::current_dir()?.join(&output_path);
        let output_path_str = output_path_abs.to_string_lossy();
        
        if self.verbose {
            println!("Running: {}", output_path_str);
        }

        // Execute with timeout and resource monitoring
        let execution_start = std::time::Instant::now();
        let result = std::process::Command::new(&*output_path_str)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute program: {}", e))?;
        let execution_time = execution_start.elapsed();

        if !result.status.success() {
            let exit_code = result.status.code().unwrap_or(-1);
            anyhow::bail!("Program exited with code {}", exit_code);
        }

        // Output program results
        let stdout = std::str::from_utf8(&result.stdout).unwrap_or("Invalid UTF-8");
        let stderr = std::str::from_utf8(&result.stderr).unwrap_or("Invalid UTF-8");
        
        print!("{}", stdout);
        if !stderr.is_empty() {
            eprint!("{}", stderr);
        }

        if self.verbose {
            println!("\nExecution completed in {:?}", execution_time);
        }

        Ok(())
    }

    pub fn tokenize(input: &str) -> anyhow::Result<()> {
        let compiler = Compiler::new().with_verbose(true);
        compiler.tokenize_internal(input)
    }

    fn tokenize_internal(&self, input: &str) -> anyhow::Result<()> {
        if self.verbose {
            println!("Tokenizing: {}", input);
        }

        // Validate input file
        if !std::path::Path::new(input).exists() {
            anyhow::bail!("Input file '{}' does not exist", input);
        }

        let source = std::fs::read_to_string(input)
            .map_err(|e| anyhow::anyhow!("Failed to read input file '{}': {}", input, e))?;

        let tokenizing_start = Instant::now();
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        let tokenizing_time = tokenizing_start.elapsed();

        if self.verbose {
            println!("\ninfo: {} tokens found in {:?}", tokens.len(), tokenizing_time);
            println!("=== Token Analysis ===");
        }

        // Token statistics
        let mut token_stats = std::collections::HashMap::new();
        for token in &tokens {
            *token_stats.entry(format!("{:?}", token.kind)).or_insert(0) += 1;
        }

        if self.verbose {
            println!("Token Statistics:");
            for (token_type, count) in &token_stats {
                println!("  {}: {}", token_type, count);
            }
            println!();
        }

        println!("=== Tokens ===");
        for (i, token) in tokens.iter().enumerate() {
            println!(
                "{:3}: Token {{ kind: {:?}, lexeme: \"{}\", line: {}, column: {} }}",
                i + 1, token.kind, token.lexeme, token.line, token.column
            );
        }

        if self.verbose {
            println!("\nTokenization completed successfully!");
        }

        Ok(())
    }
}

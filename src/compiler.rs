use std::path::PathBuf;

use crate::codegen::codegen::CodeGenerator;
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::typechecker::typechecker::TypeChecker;

const LLC_CMD: &str = "llc";
const GCC_CMD: &str = "gcc";

pub struct Compiler;

impl Compiler {
    pub fn compile(input: &str, output: Option<&str>) -> anyhow::Result<()> {
        let source = std::fs::read_to_string(input)?;

        println!("Compiling: {}", input);

        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();

        println!("info: {} tokens found", tokens.len());

        let mut parser = Parser::new(tokens);
        let program = parser
            .parse()
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

        println!("success: Parsed successfully!");
        println!("  Statements: {}", program.statements.len());

        let mut typechecker = TypeChecker::new();
        typechecker
            .check(&program)
            .map_err(|e| anyhow::anyhow!("Type error: {}", e))?;

        println!("success: Type checking passed!");

        let mut codegen = CodeGenerator::new();
        let llvm_ir = codegen.generate(&program);

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

        println!("Compiling {}...", input);

        std::fs::write(&ll_path, llvm_ir)?;

        let llc_result = std::process::Command::new(LLC_CMD)
            .arg("-filetype=obj")
            .arg("-o")
            .arg(&obj_path)
            .arg(&ll_path)
            .output()?;

        if !llc_result.status.success() {
            eprintln!("error: llc compilation failed");
            let stderr = std::str::from_utf8(&llc_result.stderr).unwrap_or("Invalid UTF-8");
            eprintln!("{}", stderr);
            return Err(anyhow::anyhow!("llc compilation failed"));
        }

        let linker_result = std::process::Command::new(GCC_CMD)
            .arg("-no-pie")
            .arg(&obj_path)
            .arg("-o")
            .arg(&output_path)
            .arg("-lc")
            .output()?;

        let _ = std::fs::remove_file(&ll_path);
        let _ = std::fs::remove_file(&obj_path);

        if linker_result.status.success() {
            println!("success: Compiled: {}", output_path.display());
        } else {
            let stderr = std::str::from_utf8(&linker_result.stderr).unwrap_or("Invalid UTF-8");
            eprintln!("\nerror details: {}", stderr);
            return Err(anyhow::anyhow!("linking failed: {}", stderr));
        }

        Ok(())
    }

    pub fn run(input: &str) -> anyhow::Result<()> {
        let input_path = PathBuf::from(input);
        let output_path = input_path.with_extension("");

        Compiler::compile(input, None)?;

        let output_path_abs = std::env::current_dir()?.join(&output_path);
        let output_path_str = output_path_abs.to_string_lossy();
        println!("Running: {}", output_path.display());

        let result = std::process::Command::new(&*output_path_str).output()?;

        if !result.status.success() {
            anyhow::bail!("error: failed to execute: {}", result.status);
        }

        print!("{}", std::str::from_utf8(&result.stdout).unwrap_or("Invalid UTF-8"));
        eprint!("{}", std::str::from_utf8(&result.stderr).unwrap_or("Invalid UTF-8"));

        Ok(())
    }

    pub fn tokenize(input: &str) -> anyhow::Result<()> {
        println!("Tokenizing: {}", input);

        let source = std::fs::read_to_string(input)?;

        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();

        println!("\ninfo: {} tokens found", tokens.len());
        println!("=== Tokens ===");
        for token in &tokens {
            println!(
                "  Token {{ kind: {:?}, lexeme: \"{}\", line: {}, column: {} }}",
                token.kind, token.lexeme, token.line, token.column
            );
        }

        Ok(())
    }
}

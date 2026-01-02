use crate::cli::Cli;

pub mod ast;
pub mod cli;
pub mod codegen;
pub mod compiler;
pub mod lexer;
pub mod ownership;
pub mod parser;
pub mod token;
pub mod typechecker;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args.get(1).is_some_and(|arg| arg == "--help" || arg == "-h") {
        Cli::print_usage();
        return;
    }

    let cli = match Cli::from_args(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {}", e);
            eprintln!();
            Cli::print_usage();
            return;
        }
    };

    if let Err(e) = cli.run() {
        eprintln!("error: {}", e);
    }
}

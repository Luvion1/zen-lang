use crate::cli::Cli;

pub mod ast;
pub mod cli;
pub mod codegen;
pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod token;
pub mod typechecker;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        Cli::print_usage();
        std::process::exit(0);
    }

    let cli = match Cli::from_args(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {}", e);
            eprintln!();
            Cli::print_usage();
            std::process::exit(1);
        }
    };

    if let Err(e) = cli.run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zen")]
#[command(about = "Zen Programming Language Compiler", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compile a Zen file to native binary
    Compile {
        /// Input Zen file
        input: String,
        /// Output file name (optional)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Compile and run a Zen file
    Run {
        /// Input Zen file
        input: String,
    },
    /// Show tokens from a Zen file
    Tokenize {
        /// Input Zen file
        input: String,
    },
}

impl Cli {
    pub fn print_usage() {
        println!("Zen Programming Language Compiler");
        println!();
        println!("Usage:");
        println!("  zen <command> [options]");
        println!();
        println!("Commands:");
        println!("  compile   Compile a Zen file to native binary");
        println!("  run       Compile and run a Zen file");
        println!("  tokenize  Show tokens from a Zen file");
        println!();
        println!("Options:");
        println!("  -o, --output <file>  Specify output file");
        println!();
        println!("Examples:");
        println!("  zen compile examples/hello.zen");
        println!("  zen compile examples/hello.zen -o /tmp/hello");
        println!("  zen run examples/hello.zen");
        println!("  zen tokenize input.zen");
    }

    pub fn from_args(args: Vec<String>) -> Result<Self, String> {
        if args.len() < 2 {
            return Err("No command specified".to_string());
        }

        Cli::try_parse_from(args).map_err(|e| e.to_string())
    }

    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            Commands::Compile { input, output } => {
                crate::compiler::Compiler::compile(&input, output.as_deref())
            }
            Commands::Run { input } => crate::compiler::Compiler::run(&input),
            Commands::Tokenize { input } => crate::compiler::Compiler::tokenize(&input),
        }
    }
}

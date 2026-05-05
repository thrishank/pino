mod commands;
mod scaffold;
mod snippets;

use clap::{Parser, Subcommand};
use colored::Colorize;

const BANNER: &str = r#"
  ██████╗ ██╗███╗   ██╗ ██████╗
  ██╔══██╗██║████╗  ██║██╔═══██╗
  ██████╔╝██║██╔██╗ ██║██║   ██║
  ██╔═══╝ ██║██║╚██╗██║██║   ██║
  ██║     ██║██║ ╚████║╚██████╔╝
  ╚═╝     ╚═╝╚═╝  ╚═══╝ ╚═════╝
"#;

#[derive(Parser)]
#[command(
    name = "pino",
    about = "Quickly initialize a Pinocchio Solana project",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Pinocchio project
    Init {
        /// Project name
        name: String,

        /// Include anchor compatibility layer
        #[arg(long, default_value_t = false)]
        anchor: bool,

        /// Include token crates (pinocchio-token, pinocchio-associated-token)
        #[arg(long, default_value_t = false)]
        token: bool,

        /// Generate IDL and TypeScript types (anchor-compatible)
        #[arg(long, default_value_t = false)]
        idl: bool,

        /// Include TypeScript test scaffold
        #[arg(long, default_value_t = false)]
        ts_tests: bool,

        /// Skip prompts and use defaults
        #[arg(long, short = 'y', default_value_t = false)]
        yes: bool,
    },

    /// Show boilerplate snippets for common operations
    Snippet {
        #[command(subcommand)]
        kind: SnippetKind,
    },

    /// List all available Pinocchio crates
    Crates,
}

#[derive(Subcommand)]
pub enum SnippetKind {
    /// How to create an account
    CreateAccount,
    /// How to transfer SOL
    Transfer,
    /// How to deserialize instruction data
    Deserialize,
    /// How to create a PDA
    Pda,
    /// How to emit a log
    Log,
    /// How to validate a pubkey
    Pubkey,
    /// List all snippets
    List,
}

fn main() {
    print_banner();
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Init {
            name,
            anchor,
            token,
            idl,
            ts_tests,
            yes,
        } => commands::init::run(name, *anchor, *token, *idl, *ts_tests, *yes),
        Commands::Snippet { kind } => commands::snippet::run(kind),
        Commands::Crates => commands::crates::run(),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn print_banner() {
    println!("{}", BANNER.bright_yellow().bold());
    println!(
        "  {} {}\n",
        "Pinocchio project scaffolder".bright_white(),
        "⚡".yellow()
    );
}

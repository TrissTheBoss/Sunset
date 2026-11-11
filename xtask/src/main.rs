#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Doctor {
        #[arg(long)]
        non_interactive: bool,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Doctor { non_interactive: _ } => {
            eprintln!("NOT IMPLEMENTED");
            ExitCode::from(2)
        }
    }
}

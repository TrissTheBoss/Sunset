#![forbid(unsafe_code)]

mod check;
mod doctor;
mod lint;

use clap::{Parser, Subcommand};
use std::path::Path;
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
    Lint {
        #[arg(long)]
        non_interactive: bool,
    },
    Check {
        #[arg(long)]
        non_interactive: bool,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Doctor { non_interactive: _ } => doctor::run(),
        Command::Lint { non_interactive: _ } => lint::run_cli(),
        Command::Check { non_interactive: _ } => check::run(),
    }
}

pub(crate) fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
}

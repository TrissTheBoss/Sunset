#![forbid(unsafe_code)]

mod check;
mod doctor;
mod inventory;
mod lint;
mod lock;

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
    Lock {
        #[arg(long)]
        non_interactive: bool,
    },
    Inventory {
        #[arg(long)]
        non_interactive: bool,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Doctor { non_interactive: _ } => doctor::run(),
        Command::Lint { non_interactive: _ } => lint::run_cli(),
        Command::Check { non_interactive: _ } => check::run(),
        Command::Lock { non_interactive: _ } => lock::run(),
        Command::Inventory { non_interactive: _ } => inventory::run(),
    }
}

pub(crate) fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
}

#[cfg(test)]
mod tests {
    #[test]
    fn bench_01_manifest_parses() {
        let path = crate::workspace_root().join("bench/scenes/bench-01.toml");
        let text = std::fs::read_to_string(path).expect("read bench-01.toml");
        let _: toml::Value = toml::from_str(&text).expect("valid TOML");
    }
}

#![forbid(unsafe_code)]

mod bench;
mod check;
mod doctor;
mod inventory;
mod lint;
mod lock;
mod runner;
mod test;

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
    Report {
        #[arg(long)]
        run: String,
    },
    Run {
        #[arg(long)]
        id: String,
        #[arg(long)]
        timeout_ms: u64,
        #[arg(long)]
        program: String,
        #[arg(long)]
        arg: Vec<String>,
    },
    Bench {
        #[arg(long)]
        suite: String,
        #[arg(long)]
        gate: bool,
        #[arg(long)]
        program: Option<String>,
        #[arg(long)]
        non_interactive: bool,
    },
    Test {
        #[arg(long)]
        suite: String,
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
        Command::Report { run } => runner::report(&run),
        Command::Run {
            id,
            timeout_ms,
            program,
            arg,
        } => runner::run_cli(&id, timeout_ms, &program, &arg),
        Command::Bench {
            suite,
            gate,
            program,
            non_interactive: _,
        } => bench::run_cli(&suite, gate, program.as_deref()),
        Command::Test {
            suite,
            non_interactive: _,
        } => test::run_cli(&suite),
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

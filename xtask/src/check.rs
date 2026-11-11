use std::process::{Command, ExitCode};

pub fn run() -> ExitCode {
    let mut failed = false;
    if !run_command("cargo", &["fmt", "--all", "--", "--check"], "fmt") {
        failed = true;
    }
    if !run_command(
        "cargo",
        &["clippy", "--all-targets", "--", "-D", "warnings"],
        "clippy",
    ) {
        failed = true;
    }
    match crate::lint::run() {
        Ok(()) => println!("check: lint ok"),
        Err(violations) => {
            failed = true;
            crate::lint::print_violations(&violations);
        }
    }
    if !run_command("cargo", &["test", "--workspace"], "test") {
        failed = true;
    }
    if failed {
        ExitCode::from(1)
    } else {
        println!("check: all steps passed");
        ExitCode::from(0)
    }
}

fn run_command(program: &str, args: &[&str], label: &str) -> bool {
    let status = Command::new(program).args(args).status();
    match status {
        Ok(status) if status.success() => {
            println!("check: {label} ok");
            true
        }
        Ok(_) => {
            eprintln!("check: {label} failed");
            false
        }
        Err(error) => {
            eprintln!("check: {label} failed to run: {error}");
            false
        }
    }
}

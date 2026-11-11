use command_group::CommandGroup;
use serde::Serialize;
use std::io::Read;
use std::process::{Command, ExitCode, ExitStatus, Stdio};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunOutcome {
    Passed,
    Failed(i32),
    Crashed,
    TimedOut,
    MissingPrerequisite,
    IoError,
}

impl RunOutcome {
    pub fn exit_code(self) -> u8 {
        match self {
            RunOutcome::Passed => 0,
            RunOutcome::Failed(_) => 1,
            RunOutcome::Crashed => 3,
            RunOutcome::TimedOut => 4,
            RunOutcome::MissingPrerequisite => 2,
            RunOutcome::IoError => 3,
        }
    }
}

#[derive(Serialize)]
pub struct RunReport {
    pub run_id: String,
    pub program: String,
    pub args: Vec<String>,
    pub outcome: String,
    pub exit_code: u8,
    pub duration_ms: u128,
    pub timestamp: u64,
}

pub struct CapturedOutput {
    pub outcome: RunOutcome,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

fn spawn_group(command: &mut Command) -> Result<command_group::GroupChild, std::io::Error> {
    let mut attempt = 0u32;
    loop {
        match command.group_spawn() {
            Ok(child) => return Ok(child),
            Err(error) if is_transient(&error) => {
                attempt += 1;
                if attempt >= 3 {
                    return Err(error);
                }
                std::thread::sleep(Duration::from_millis(25 * u64::from(attempt)));
            }
            Err(error) => return Err(error),
        }
    }
}

fn is_transient(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::WouldBlock
            | std::io::ErrorKind::Interrupted
            | std::io::ErrorKind::ExecutableFileBusy
    )
}

pub fn run_captured(program: &str, args: &[String], timeout: Duration) -> CapturedOutput {
    let mut command = Command::new(program);
    command.args(args);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = match spawn_group(&mut command) {
        Ok(child) => child,
        Err(error) => {
            let outcome = if error.kind() == std::io::ErrorKind::NotFound {
                RunOutcome::MissingPrerequisite
            } else {
                RunOutcome::IoError
            };
            return CapturedOutput {
                outcome,
                stdout: Vec::new(),
                stderr: Vec::new(),
            };
        }
    };
    let stdout_handle = child.inner().stdout.take().map(spawn_reader);
    let stderr_handle = child.inner().stderr.take().map(spawn_reader);
    let deadline = Instant::now() + timeout;
    let outcome = loop {
        match child.try_wait() {
            Ok(Some(status)) => break classify_status(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break RunOutcome::TimedOut;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break RunOutcome::IoError;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    CapturedOutput {
        outcome,
        stdout: join_reader(stdout_handle),
        stderr: join_reader(stderr_handle),
    }
}

fn spawn_reader<R: Read + Send + 'static>(mut stream: R) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stream.read_to_end(&mut buffer);
        buffer
    })
}

fn join_reader(handle: Option<std::thread::JoinHandle<Vec<u8>>>) -> Vec<u8> {
    handle
        .map(|h| h.join().unwrap_or_default())
        .unwrap_or_default()
}

pub fn run_child(program: &str, args: &[String], timeout: Duration) -> RunOutcome {
    let mut command = Command::new(program);
    command.args(args);
    let mut child = match spawn_group(&mut command) {
        Ok(child) => child,
        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                return RunOutcome::MissingPrerequisite;
            }
            return RunOutcome::IoError;
        }
    };
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return classify_status(status),
            Ok(None) => {}
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return RunOutcome::IoError,
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return RunOutcome::TimedOut;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn run_recorded(run_id: &str, program: &str, args: &[String], timeout: Duration) -> RunReport {
    let start = Instant::now();
    let outcome = run_child(program, args, timeout);
    RunReport {
        run_id: run_id.to_string(),
        program: program.to_string(),
        args: args.to_vec(),
        outcome: format!("{outcome:?}"),
        exit_code: outcome.exit_code(),
        duration_ms: start.elapsed().as_millis(),
        timestamp: unix_seconds(),
    }
}

pub fn run_cli(id: &str, timeout_ms: u64, program: &str, args: &[String]) -> ExitCode {
    let report = run_recorded(id, program, args, Duration::from_millis(timeout_ms));
    let _ = write_run(&report);
    ExitCode::from(report.exit_code)
}

pub fn write_run(report: &RunReport) -> std::io::Result<()> {
    let directory = crate::workspace_root().join("runs").join(&report.run_id);
    std::fs::create_dir_all(&directory)?;
    let json = serde_json::to_string_pretty(report)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    std::fs::write(directory.join("report.json"), json)?;
    std::fs::write(directory.join("report.md"), markdown(report))?;
    Ok(())
}

pub fn report(run_id: &str) -> ExitCode {
    let path = crate::workspace_root()
        .join("runs")
        .join(run_id)
        .join("report.json");
    match std::fs::read_to_string(&path) {
        Ok(json) => {
            println!("{json}");
            ExitCode::from(0)
        }
        Err(_) => {
            eprintln!("no run recorded for id {run_id}");
            ExitCode::from(2)
        }
    }
}

#[cfg(unix)]
fn classify_status(status: ExitStatus) -> RunOutcome {
    use std::os::unix::process::ExitStatusExt;
    if status.success() {
        RunOutcome::Passed
    } else if status.signal().is_some() {
        RunOutcome::Crashed
    } else {
        RunOutcome::Failed(status.code().unwrap_or(1))
    }
}

#[cfg(not(unix))]
fn classify_status(status: ExitStatus) -> RunOutcome {
    if status.success() {
        RunOutcome::Passed
    } else {
        RunOutcome::Failed(status.code().unwrap_or(1))
    }
}

fn markdown(report: &RunReport) -> String {
    format!(
        "# Run {}\n\n- program: {}\n- args: {}\n- outcome: {}\n- exit code: {}\n- duration: {} ms\n- timestamp: {}\n",
        report.run_id,
        report.program,
        report.args.join(" "),
        report.outcome,
        report.exit_code,
        report.duration_ms,
        report.timestamp
    )
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_match_runner_contract() {
        assert_eq!(RunOutcome::Passed.exit_code(), 0);
        assert_eq!(RunOutcome::Failed(1).exit_code(), 1);
        assert_eq!(RunOutcome::MissingPrerequisite.exit_code(), 2);
        assert_eq!(RunOutcome::Crashed.exit_code(), 3);
        assert_eq!(RunOutcome::IoError.exit_code(), 3);
        assert_eq!(RunOutcome::TimedOut.exit_code(), 4);
    }

    #[test]
    fn hanging_child_times_out() {
        let outcome = run_child("sleep", &["30".to_string()], Duration::from_millis(100));
        assert_eq!(outcome, RunOutcome::TimedOut);
    }

    #[test]
    fn crashing_child_reports_crash() {
        let outcome = run_child(
            "sh",
            &["-c".to_string(), "kill -SEGV $$".to_string()],
            Duration::from_secs(5),
        );
        assert_eq!(outcome, RunOutcome::Crashed);
    }

    #[test]
    fn missing_prerequisite_reports_blocked() {
        let outcome = run_child(
            "/nonexistent/definitely/not/a/binary",
            &[],
            Duration::from_secs(1),
        );
        assert_eq!(outcome, RunOutcome::MissingPrerequisite);
    }

    #[test]
    fn killing_group_removes_descendants() {
        let pid_file = "/tmp/sunset_grandchild.pid";
        let _ = std::fs::remove_file(pid_file);
        let outcome = run_child(
            "sh",
            &[
                "-c".to_string(),
                format!("sleep 60 & echo $! > {pid_file}; sleep 60"),
            ],
            Duration::from_secs(1),
        );
        assert_eq!(outcome, RunOutcome::TimedOut);
        std::thread::sleep(Duration::from_millis(200));
        if let Ok(text) = std::fs::read_to_string(pid_file) {
            if let Ok(pid) = text.trim().parse::<i32>() {
                let alive = Command::new("kill")
                    .args(["-0", &pid.to_string()])
                    .status()
                    .map(|status| status.success())
                    .unwrap_or(false);
                assert!(!alive, "grandchild {pid} survived the group kill");
            }
        }
    }
}

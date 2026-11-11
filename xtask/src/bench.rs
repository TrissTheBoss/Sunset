use crate::runner::{run_captured, RunOutcome};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

const PERF_T1_MEDIAN_FPS: f64 = 3_000.0;
const PERF_T1_MIN_FPS: f64 = 2_850.0;
const PERF_T2_P99_MS: f64 = 0.50;
const PERF_T2_P99_9_MS: f64 = 1.00;
const PERF_T2_OVER_MS: f32 = 16.67;
const NOISE_THRESHOLD: f64 = 0.03;

#[derive(Deserialize)]
struct Metrics {
    schema: u32,
    scene: String,
    frames: u64,
    elapsed_s: f64,
    present_mode: String,
    intervals_ms: Vec<f32>,
}

#[derive(Deserialize)]
struct SceneManifest {
    scene: Scene,
    timing: Timing,
}

#[derive(Deserialize)]
struct Scene {
    name: String,
}

#[derive(Deserialize)]
struct Timing {
    runs: usize,
    #[serde(default)]
    warmup_s: f64,
    #[serde(default)]
    sampling_s: f64,
}

#[derive(Clone, Debug)]
struct RunMetrics {
    throughput_fps: f64,
    present_mode: String,
    p99_ms: f64,
    p99_9_ms: f64,
    over_16_67_ms: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum GateVerdict {
    Pass,
    Fail(Vec<String>),
    Inconclusive(String),
}

impl GateVerdict {
    fn label(&self) -> &'static str {
        match self {
            GateVerdict::Pass => "pass",
            GateVerdict::Fail(_) => "fail",
            GateVerdict::Inconclusive(_) => "inconclusive",
        }
    }

    fn reasons(&self) -> Vec<String> {
        match self {
            GateVerdict::Pass => Vec::new(),
            GateVerdict::Fail(reasons) => reasons.clone(),
            GateVerdict::Inconclusive(reason) => vec![reason.clone()],
        }
    }

    fn exit_code(&self) -> u8 {
        match self {
            GateVerdict::Pass => 0,
            GateVerdict::Fail(_) | GateVerdict::Inconclusive(_) => 1,
        }
    }
}

#[derive(Serialize)]
struct BenchReport {
    schema: u32,
    run_id: String,
    suite: String,
    program: String,
    gate: bool,
    verdict: String,
    exit_code: u8,
    runs_requested: usize,
    runs_completed: usize,
    run_results: Vec<RunResult>,
    reasons: Vec<String>,
    timestamp: u64,
}

#[derive(Serialize)]
struct RunResult {
    index: usize,
    outcome: String,
    throughput_fps: Option<f64>,
    p99_ms: Option<f64>,
    p99_9_ms: Option<f64>,
    over_16_67_ms: Option<usize>,
    present_mode: Option<String>,
    detail: String,
    stderr: String,
}

impl RunResult {
    fn empty(index: usize, outcome: &str, detail: String, stderr: String) -> Self {
        Self {
            index,
            outcome: outcome.to_string(),
            throughput_fps: None,
            p99_ms: None,
            p99_9_ms: None,
            over_16_67_ms: None,
            present_mode: None,
            detail,
            stderr,
        }
    }

    fn from_metrics(index: usize, metrics: &RunMetrics) -> Self {
        Self {
            index,
            outcome: "passed".to_string(),
            throughput_fps: Some(metrics.throughput_fps),
            p99_ms: Some(metrics.p99_ms),
            p99_9_ms: Some(metrics.p99_9_ms),
            over_16_67_ms: Some(metrics.over_16_67_ms),
            present_mode: Some(metrics.present_mode.clone()),
            detail: String::new(),
            stderr: String::new(),
        }
    }
}

pub fn run_cli(suite: &str, gate: bool, program_override: Option<&str>) -> ExitCode {
    let manifest_path = crate::workspace_root()
        .join("bench")
        .join("scenes")
        .join(format!("{suite}.toml"));
    let (report, exit_code) = run_bench(&manifest_path, program_override, gate);
    let _ = write_report(&report);
    match report.verdict.as_str() {
        "pass" => println!("bench: completed"),
        "fail" => eprintln!("bench: failed: {}", report.reasons.join("; ")),
        "inconclusive" => eprintln!("bench: INCONCLUSIVE: {}", report.reasons.join("; ")),
        "blocked" => eprintln!("bench: BLOCKED: {}", report.reasons.join("; ")),
        other => eprintln!("bench: finished with verdict {other}"),
    }
    ExitCode::from(exit_code)
}

fn run_bench(
    manifest_path: &Path,
    program_override: Option<&str>,
    gate: bool,
) -> (BenchReport, u8) {
    let run_id = timestamp().to_string();
    let suite = manifest_path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

    let manifest_text = match fs::read_to_string(manifest_path) {
        Ok(text) => text,
        Err(error) => {
            return (
                blocked_report(
                    &run_id,
                    &suite,
                    program_override,
                    gate,
                    format!("cannot read manifest: {error}"),
                ),
                2,
            );
        }
    };
    let manifest: SceneManifest = match toml::from_str(&manifest_text) {
        Ok(value) => value,
        Err(error) => {
            return (
                blocked_report(
                    &run_id,
                    &suite,
                    program_override,
                    gate,
                    format!("invalid manifest: {error}"),
                ),
                2,
            );
        }
    };
    if !manifest.scene.name.eq_ignore_ascii_case(&suite) {
        return (
            blocked_report(
                &run_id,
                &suite,
                program_override,
                gate,
                format!(
                    "scene name '{}' does not match suite '{}'",
                    manifest.scene.name, suite
                ),
            ),
            2,
        );
    }
    let program = match program_override {
        Some(path) => path.to_string(),
        None => crate::workspace_root()
            .join("target/release/sunset-client")
            .to_string_lossy()
            .into_owned(),
    };
    if !Path::new(&program).exists() {
        let hint = if program_override.is_some() {
            "check --program"
        } else {
            "build target/release/sunset-client (M2) or pass --program"
        };
        return (
            blocked_report(
                &run_id,
                &suite,
                Some(&program),
                gate,
                format!("client binary not found: {program}; {hint}"),
            ),
            2,
        );
    }

    let timeout =
        Duration::from_secs_f64(manifest.timing.warmup_s + manifest.timing.sampling_s + 30.0);

    let mut run_results = Vec::new();
    let mut run_metrics = Vec::new();
    let mut first_hard_exit: Option<u8> = None;
    let mut had_soft_failure = false;

    for index in 0..manifest.timing.runs {
        let args = vec![
            manifest_path.to_string_lossy().into_owned(),
            index.to_string(),
        ];
        let captured = run_captured(&program, &args, timeout);
        let stderr = String::from_utf8_lossy(&captured.stderr).into_owned();
        match captured.outcome {
            RunOutcome::Passed => match parse_metrics(&captured.stdout, &suite) {
                Ok(metrics) => {
                    let run_metric = compute_metrics(&metrics);
                    run_results.push(RunResult::from_metrics(index, &run_metric));
                    run_metrics.push(run_metric);
                }
                Err(error) => {
                    had_soft_failure = true;
                    run_results.push(RunResult::empty(index, "invalid-metrics", error, stderr));
                }
            },
            RunOutcome::TimedOut => {
                if first_hard_exit.is_none() {
                    first_hard_exit = Some(4);
                }
                run_results.push(RunResult::empty(index, "timed_out", String::new(), stderr));
            }
            RunOutcome::Crashed => {
                if first_hard_exit.is_none() {
                    first_hard_exit = Some(3);
                }
                run_results.push(RunResult::empty(index, "crashed", String::new(), stderr));
            }
            RunOutcome::IoError => {
                if first_hard_exit.is_none() {
                    first_hard_exit = Some(3);
                }
                run_results.push(RunResult::empty(index, "io-error", String::new(), stderr));
            }
            RunOutcome::MissingPrerequisite => {
                if first_hard_exit.is_none() {
                    first_hard_exit = Some(2);
                }
                run_results.push(RunResult::empty(
                    index,
                    "missing-prerequisite",
                    String::new(),
                    stderr,
                ));
            }
            RunOutcome::Failed(code) => {
                had_soft_failure = true;
                run_results.push(RunResult::empty(
                    index,
                    "failed",
                    format!("exit code {code}"),
                    stderr,
                ));
            }
        }
    }

    let (verdict, exit_code) = if let Some(code) = first_hard_exit {
        (
            GateVerdict::Fail(vec![format!("a run did not complete (runner exit {code})")]),
            code,
        )
    } else if had_soft_failure {
        (
            GateVerdict::Fail(vec![
                "one or more runs did not produce valid metrics".to_string()
            ]),
            1,
        )
    } else if gate {
        let verdict = evaluate_gate(&run_metrics);
        let code = verdict.exit_code();
        (verdict, code)
    } else {
        (GateVerdict::Pass, 0)
    };

    let report = BenchReport {
        schema: 1,
        run_id,
        suite,
        program,
        gate,
        verdict: verdict.label().to_string(),
        exit_code,
        runs_requested: manifest.timing.runs,
        runs_completed: run_metrics.len(),
        run_results,
        reasons: verdict.reasons(),
        timestamp: timestamp(),
    };
    (report, exit_code)
}

fn blocked_report(
    run_id: &str,
    suite: &str,
    program: Option<&str>,
    gate: bool,
    message: String,
) -> BenchReport {
    BenchReport {
        schema: 1,
        run_id: run_id.to_string(),
        suite: suite.to_string(),
        program: program.unwrap_or("(unset)").to_string(),
        gate,
        verdict: "blocked".to_string(),
        exit_code: 2,
        runs_requested: 0,
        runs_completed: 0,
        run_results: Vec::new(),
        reasons: vec![message],
        timestamp: timestamp(),
    }
}

fn parse_metrics(bytes: &[u8], expected_scene: &str) -> Result<Metrics, String> {
    let metrics: Metrics =
        serde_json::from_slice(bytes).map_err(|error| format!("invalid metrics JSON: {error}"))?;
    if metrics.schema != 1 {
        return Err(format!("unsupported metrics schema {}", metrics.schema));
    }
    if !metrics.scene.eq_ignore_ascii_case(expected_scene) {
        return Err(format!(
            "metrics scene '{}' does not match suite '{}'",
            metrics.scene, expected_scene
        ));
    }
    Ok(metrics)
}

fn compute_metrics(metrics: &Metrics) -> RunMetrics {
    let throughput_fps = if metrics.elapsed_s > 0.0 {
        metrics.frames as f64 / metrics.elapsed_s
    } else {
        0.0
    };
    let over_16_67_ms = metrics
        .intervals_ms
        .iter()
        .filter(|value| **value > PERF_T2_OVER_MS)
        .count();
    let p99_ms = sunset_metrics::percentile(&metrics.intervals_ms, 0.99).unwrap_or(0.0) as f64;
    let p99_9_ms = sunset_metrics::percentile(&metrics.intervals_ms, 0.999).unwrap_or(0.0) as f64;
    RunMetrics {
        throughput_fps,
        present_mode: metrics.present_mode.clone(),
        p99_ms,
        p99_9_ms,
        over_16_67_ms,
    }
}

fn evaluate_gate(runs: &[RunMetrics]) -> GateVerdict {
    if runs.is_empty() {
        return GateVerdict::Fail(vec!["no completed runs to evaluate".to_string()]);
    }
    if let Some(run) = runs
        .iter()
        .find(|run| run.present_mode.eq_ignore_ascii_case("fifo"))
    {
        return GateVerdict::Fail(vec![format!(
            "present mode Fifo invalidates the gated run (mode {})",
            run.present_mode
        )]);
    }
    if let Some(reason) = noise_reason(runs) {
        return GateVerdict::Inconclusive(reason);
    }

    let mut reasons = Vec::new();
    let throughputs: Vec<f64> = runs.iter().map(|run| run.throughput_fps).collect();
    let median = median_fps(&throughputs);
    let minimum = throughputs.iter().cloned().fold(f64::INFINITY, f64::min);
    if median < PERF_T1_MEDIAN_FPS {
        reasons.push(format!(
            "PERF-T1: median {median:.1} FPS < {PERF_T1_MEDIAN_FPS:.0} FPS"
        ));
    }
    if minimum < PERF_T1_MIN_FPS {
        reasons.push(format!(
            "PERF-T1: minimum {minimum:.1} FPS < {PERF_T1_MIN_FPS:.0} FPS"
        ));
    }
    for run in runs {
        if run.p99_ms > PERF_T2_P99_MS {
            reasons.push(format!(
                "PERF-T2: p99 {:.3} ms > {:.2} ms",
                run.p99_ms, PERF_T2_P99_MS
            ));
        }
        if run.p99_9_ms > PERF_T2_P99_9_MS {
            reasons.push(format!(
                "PERF-T2: p99.9 {:.3} ms > {:.2} ms",
                run.p99_9_ms, PERF_T2_P99_9_MS
            ));
        }
        if run.over_16_67_ms > 0 {
            reasons.push(format!(
                "PERF-T2: {} interval(s) > 16.67 ms",
                run.over_16_67_ms
            ));
        }
    }
    if reasons.is_empty() {
        GateVerdict::Pass
    } else {
        GateVerdict::Fail(reasons)
    }
}

fn median_fps(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let length = sorted.len();
    if length == 0 {
        return 0.0;
    }
    let mid = length / 2;
    if length.is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

fn noise_reason(runs: &[RunMetrics]) -> Option<String> {
    if runs.len() < 2 {
        return None;
    }
    let values: Vec<f64> = runs.iter().map(|run| run.throughput_fps).collect();
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    if mean <= 0.0 {
        return None;
    }
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;
    let coefficient_of_variation = variance.sqrt() / mean;
    if coefficient_of_variation > NOISE_THRESHOLD {
        Some(format!(
            "run-to-run throughput noise (CV {:.2}%) exceeds {:.1}% threshold",
            coefficient_of_variation * 100.0,
            NOISE_THRESHOLD * 100.0
        ))
    } else {
        None
    }
}

fn write_report(report: &BenchReport) -> std::io::Result<()> {
    let directory = crate::workspace_root().join("runs").join(&report.run_id);
    fs::create_dir_all(&directory)?;
    let json = serde_json::to_string_pretty(report)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    fs::write(directory.join("bench.json"), json)?;
    fs::write(directory.join("bench.md"), markdown(report))?;
    Ok(())
}

fn markdown(report: &BenchReport) -> String {
    let mut output = String::new();
    output.push_str("# Bench report\n\n");
    output.push_str(&format!("- schema: {}\n", report.schema));
    output.push_str(&format!("- run id: {}\n", report.run_id));
    output.push_str(&format!("- suite: {}\n", report.suite));
    output.push_str(&format!("- program: {}\n", report.program));
    output.push_str(&format!("- gate: {}\n", report.gate));
    output.push_str(&format!("- verdict: {}\n", report.verdict));
    output.push_str(&format!("- exit code: {}\n", report.exit_code));
    output.push_str(&format!(
        "- runs: {} requested / {} completed\n",
        report.runs_requested, report.runs_completed
    ));
    for reason in &report.reasons {
        output.push_str(&format!("- reason: {reason}\n"));
    }
    output
}

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn stable(throughput_fps: f64, p99_ms: f64, p99_9_ms: f64, over: usize) -> RunMetrics {
        RunMetrics {
            throughput_fps,
            present_mode: "Immediate".to_string(),
            p99_ms,
            p99_9_ms,
            over_16_67_ms: over,
        }
    }

    #[test]
    fn gate_passes_at_3100_fps() {
        let runs = vec![
            stable(3100.0, 0.35, 0.40, 0),
            stable(3110.0, 0.35, 0.40, 0),
            stable(3090.0, 0.35, 0.40, 0),
            stable(3105.0, 0.35, 0.40, 0),
            stable(3095.0, 0.35, 0.40, 0),
        ];
        assert_eq!(evaluate_gate(&runs), GateVerdict::Pass);
    }

    #[test]
    fn gate_fails_perf_t1_at_2900_fps() {
        let runs = vec![
            stable(2900.0, 0.35, 0.40, 0),
            stable(2910.0, 0.35, 0.40, 0),
            stable(2890.0, 0.35, 0.40, 0),
            stable(2905.0, 0.35, 0.40, 0),
            stable(2895.0, 0.35, 0.40, 0),
        ];
        let verdict = evaluate_gate(&runs);
        assert!(matches!(verdict, GateVerdict::Fail(_)));
        let reasons = verdict.reasons();
        assert!(reasons.iter().any(|r| r.starts_with("PERF-T1")));
    }

    #[test]
    fn gate_fails_perf_t2_on_slow_intervals() {
        let runs = vec![
            stable(3100.0, 0.60, 0.40, 0),
            stable(3100.0, 0.35, 1.20, 0),
            stable(3100.0, 0.35, 0.40, 1),
            stable(3100.0, 0.35, 0.40, 0),
            stable(3100.0, 0.35, 0.40, 0),
        ];
        let verdict = evaluate_gate(&runs);
        assert!(matches!(verdict, GateVerdict::Fail(_)));
        let reasons = verdict.reasons();
        assert!(reasons.iter().any(|r| r.starts_with("PERF-T2")));
    }

    #[test]
    fn noisy_runs_are_inconclusive() {
        let runs = vec![
            stable(3100.0, 0.35, 0.40, 0),
            stable(2900.0, 0.35, 0.40, 0),
            stable(3100.0, 0.35, 0.40, 0),
            stable(2900.0, 0.35, 0.40, 0),
            stable(3100.0, 0.35, 0.40, 0),
        ];
        assert!(matches!(evaluate_gate(&runs), GateVerdict::Inconclusive(_)));
    }

    #[test]
    fn fifo_present_mode_invalidates_gate() {
        let mut runs = vec![
            stable(3100.0, 0.35, 0.40, 0),
            stable(3100.0, 0.35, 0.40, 0),
            stable(3100.0, 0.35, 0.40, 0),
            stable(3100.0, 0.35, 0.40, 0),
            stable(3100.0, 0.35, 0.40, 0),
        ];
        runs[0].present_mode = "Fifo".to_string();
        assert!(matches!(evaluate_gate(&runs), GateVerdict::Fail(_)));
    }

    #[test]
    fn metrics_json_parses_and_computes_throughput() {
        let json = br#"{"schema":1,"scene":"bench-01","frames":310000,"elapsed_s":100.0,"present_mode":"Immediate","intervals_ms":[0.32,0.32,0.33,0.34,0.32]}"#;
        let metrics = parse_metrics(json, "bench-01").expect("valid metrics");
        let run = compute_metrics(&metrics);
        assert!((run.throughput_fps - 3100.0).abs() < 0.001);
        assert_eq!(run.present_mode, "Immediate");
        assert!(run.p99_ms <= 0.5);
        assert_eq!(run.over_16_67_ms, 0);
    }

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("sunset-bench-{tag}-{}", std::process::id()));
        fs::create_dir_all(&path).expect("temp dir");
        path
    }

    fn write_fake_client(dir: &Path, body: &str) -> std::path::PathBuf {
        let path = dir.join("fake-client");
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write client");
        let mut permissions = fs::metadata(&path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("chmod");
        path
    }

    fn write_manifest(dir: &Path, runs: usize) -> std::path::PathBuf {
        let path = dir.join("bench-01.toml");
        fs::write(
            &path,
            format!("[scene]\nname = \"bench-01\"\n\n[timing]\nruns = {runs}\n"),
        )
        .expect("write manifest");
        path
    }

    #[test]
    fn fake_client_at_2900_fails_gate() {
        let dir = temp_dir("2900");
        let client = write_fake_client(
            &dir,
            r#"echo '{"schema":1,"scene":"bench-01","frames":290000,"elapsed_s":100.0,"present_mode":"Immediate","intervals_ms":[0.34,0.34,0.34,0.34,0.34]}'"#,
        );
        let manifest = write_manifest(&dir, 1);
        let (report, exit_code) = run_bench(&manifest, Some(client.to_str().unwrap()), true);
        assert_eq!(exit_code, 1);
        assert_eq!(report.verdict, "fail");
        assert!(report.reasons.iter().any(|r| r.starts_with("PERF-T1")));
    }

    #[test]
    fn fake_client_at_3100_passes_gate() {
        let dir = temp_dir("3100");
        let client = write_fake_client(
            &dir,
            r#"echo '{"schema":1,"scene":"bench-01","frames":310000,"elapsed_s":100.0,"present_mode":"Immediate","intervals_ms":[0.32,0.32,0.32,0.32,0.32]}'"#,
        );
        let manifest = write_manifest(&dir, 1);
        let (report, exit_code) = run_bench(&manifest, Some(client.to_str().unwrap()), true);
        assert_eq!(exit_code, 0);
        assert_eq!(report.verdict, "pass");
    }

    #[test]
    fn noisy_fake_client_is_inconclusive() {
        let dir = temp_dir("noisy");
        let client = write_fake_client(
            &dir,
            "case \"$2\" in\n  0|2|4) echo '{\"schema\":1,\"scene\":\"bench-01\",\"frames\":310000,\"elapsed_s\":100.0,\"present_mode\":\"Immediate\",\"intervals_ms\":[0.32,0.32,0.32]}' ;;\n  1|3) echo '{\"schema\":1,\"scene\":\"bench-01\",\"frames\":290000,\"elapsed_s\":100.0,\"present_mode\":\"Immediate\",\"intervals_ms\":[0.34,0.34,0.34]}' ;;\nesac",
        );
        let manifest = write_manifest(&dir, 5);
        let (report, exit_code) = run_bench(&manifest, Some(client.to_str().unwrap()), true);
        assert_eq!(exit_code, 1);
        assert_eq!(report.verdict, "inconclusive");
    }
}

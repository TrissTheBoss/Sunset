use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

pub enum Outcome {
    Pass,
    Blocked(String),
    Fail(String),
}

fn exit_code(outcome: &Outcome) -> u8 {
    match outcome {
        Outcome::Pass => 0,
        Outcome::Blocked(_) => 2,
        Outcome::Fail(_) => 1,
    }
}

#[derive(Serialize)]
struct DoctorReport {
    schema: u32,
    run_id: String,
    timestamp: u64,
    commit: Option<String>,
    os: OsInfo,
    cpu: CpuInfo,
    memory: MemoryInfo,
    rust: RustInfo,
    gpu: GpuInfo,
    release_profile: ReleaseProfile,
    private_root: PrivateRoot,
    launcher: Launcher,
}

#[derive(Serialize)]
struct OsInfo {
    name: String,
    version: String,
}

#[derive(Serialize)]
struct CpuInfo {
    model: String,
    physical_cores: usize,
    logical_threads: usize,
}

#[derive(Serialize)]
struct MemoryInfo {
    total_bytes: Option<u64>,
}

#[derive(Serialize)]
struct RustInfo {
    version: Option<String>,
}

#[derive(Serialize)]
struct GpuAdapter {
    name: String,
    backend: String,
    device_type: String,
    driver: String,
    driver_info: String,
}

#[derive(Serialize)]
struct GpuInfo {
    available: bool,
    adapters: Vec<GpuAdapter>,
}

#[derive(Serialize)]
struct ReleaseProfile {
    opt_level: Option<String>,
    lto: Option<String>,
    codegen_units: Option<String>,
    panic: Option<String>,
    debug: Option<String>,
    matches_perf_a10: bool,
}

#[derive(Serialize)]
struct PrivateRoot {
    path: String,
    present: bool,
}

#[derive(Serialize)]
struct Launcher {
    found: bool,
    installs: Vec<String>,
}

pub fn run() -> ExitCode {
    let report = collect_report();
    let outcome = evaluate(&report);
    let _ = write_report(&report);
    match &outcome {
        Outcome::Blocked(message) | Outcome::Fail(message) => eprintln!("{message}"),
        Outcome::Pass => println!("doctor: all required checks passed"),
    }
    ExitCode::from(exit_code(&outcome))
}

fn evaluate(report: &DoctorReport) -> Outcome {
    let mut blockers = Vec::new();
    let mut failures = Vec::new();
    if report.rust.version.is_none() {
        blockers.push("Rust toolchain not found (`rustc --version` failed).".to_string());
    }
    if !report.launcher.found {
        blockers.push(
            "No Minecraft installation found; install Java Edition 26.3 via a supported launcher."
                .to_string(),
        );
    }
    if !report.release_profile.matches_perf_a10 {
        failures.push(
            "Workspace [profile.release] does not match PERF-A10 (opt-level=3, lto=fat, \
             codegen-units=1, panic=abort, debug=line-tables-only)."
                .to_string(),
        );
    }
    if !blockers.is_empty() {
        Outcome::Blocked(blockers.join(" "))
    } else if !failures.is_empty() {
        Outcome::Fail(failures.join(" "))
    } else {
        Outcome::Pass
    }
}

fn collect_report() -> DoctorReport {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let run_id = format!("{timestamp}");
    DoctorReport {
        schema: 1,
        run_id,
        timestamp,
        commit: detect_commit(),
        os: detect_os(),
        cpu: detect_cpu(),
        memory: detect_memory(),
        rust: detect_rust(),
        gpu: detect_gpu(),
        release_profile: detect_release_profile(),
        private_root: detect_private_root(),
        launcher: detect_launcher(),
    }
}

fn detect_commit() -> Option<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn detect_os() -> OsInfo {
    let name = format!("{}-{}", env::consts::OS, env::consts::ARCH);
    let version = fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                let line = line.trim();
                line.strip_prefix("PRETTY_NAME=")
                    .map(|value| value.trim_matches('"').to_string())
            })
        })
        .unwrap_or_else(|| "UNKNOWN".to_string());
    OsInfo { name, version }
}

fn detect_cpu() -> CpuInfo {
    let logical_threads = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(0);
    let text = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let model = text
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            let key = key.trim();
            if key == "model name" || key == "Hardware" || key == "Model" {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "UNKNOWN".to_string());
    let physical_cores = text
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            if key.trim() == "cpu cores" {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(logical_threads);
    CpuInfo {
        model,
        physical_cores,
        logical_threads,
    }
}

fn detect_memory() -> MemoryInfo {
    let text = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let total_kb = text.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.trim() == "MemTotal" {
            value.split_whitespace().next()?.parse::<u64>().ok()
        } else {
            None
        }
    });
    MemoryInfo {
        total_bytes: total_kb.map(|kb| kb * 1024),
    }
}

fn detect_rust() -> RustInfo {
    let version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());
    RustInfo { version }
}

fn detect_gpu() -> GpuInfo {
    let adapters = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = wgpu::Backends::all();
        let instance = wgpu::Instance::new(descriptor);
        pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()))
    }));
    match adapters {
        Ok(list) => {
            let mapped = list
                .iter()
                .map(|adapter| {
                    let info = adapter.get_info();
                    let backend = format!("{:?}", info.backend);
                    let device_type = format!("{:?}", info.device_type);
                    GpuAdapter {
                        name: info.name,
                        backend,
                        device_type,
                        driver: info.driver,
                        driver_info: info.driver_info,
                    }
                })
                .collect();
            GpuInfo {
                available: true,
                adapters: mapped,
            }
        }
        Err(_) => GpuInfo {
            available: false,
            adapters: Vec::new(),
        },
    }
}

fn detect_release_profile() -> ReleaseProfile {
    let workspace = crate::workspace_root();
    let text = fs::read_to_string(workspace.join("Cargo.toml")).unwrap_or_default();
    let mut values = BTreeMap::new();
    let mut in_section = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[profile.release]" {
            in_section = true;
            continue;
        }
        if in_section && trimmed.starts_with('[') {
            break;
        }
        if in_section {
            if let Some((key, value)) = trimmed.split_once('=') {
                values.insert(
                    key.trim().to_string(),
                    value.trim().trim_matches('"').to_string(),
                );
            }
        }
    }
    let get = |key: &str| values.get(key).cloned();
    let opt_level = get("opt-level");
    let lto = get("lto");
    let codegen_units = get("codegen-units");
    let panic = get("panic");
    let debug = get("debug");
    let matches = opt_level.as_deref() == Some("3")
        && lto.as_deref() == Some("fat")
        && codegen_units.as_deref() == Some("1")
        && panic.as_deref() == Some("abort")
        && debug.as_deref() == Some("line-tables-only");
    ReleaseProfile {
        opt_level,
        lto,
        codegen_units,
        panic,
        debug,
        matches_perf_a10: matches,
    }
}

fn detect_private_root() -> PrivateRoot {
    let workspace = crate::workspace_root();
    let path = env::var("SUNSET_PRIVATE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            workspace
                .parent()
                .unwrap_or(workspace)
                .join("sunset-private")
        });
    let present = path.is_dir();
    PrivateRoot {
        path: path.to_string_lossy().into_owned(),
        present,
    }
}

fn detect_launcher() -> Launcher {
    let home = env::var("HOME").map(PathBuf::from).unwrap_or_default();
    let mut installs = Vec::new();
    let vanilla = home.join(".minecraft");
    if vanilla.join("versions").is_dir() || vanilla.join("assets").is_dir() {
        installs.push(vanilla.to_string_lossy().into_owned());
    }
    let instance_roots = [
        home.join(".var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances"),
        home.join(".local/share/PrismLauncher/instances"),
        home.join(".local/share/multimc/instances"),
        home.join(".var/app/org.multimc.MultiMC/data/MultiMC/instances"),
    ];
    for root in instance_roots {
        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                let minecraft = entry.path().join("minecraft");
                if minecraft.is_dir() {
                    installs.push(minecraft.to_string_lossy().into_owned());
                }
            }
        }
    }
    installs.sort();
    installs.dedup();
    Launcher {
        found: !installs.is_empty(),
        installs,
    }
}

fn write_report(report: &DoctorReport) -> std::io::Result<()> {
    let directory = Path::new("runs").join(&report.run_id);
    fs::create_dir_all(&directory)?;
    let json = serde_json::to_string_pretty(report)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    fs::write(directory.join("doctor.json"), json)?;
    fs::write(directory.join("doctor.md"), markdown(report))?;
    Ok(())
}

fn markdown(report: &DoctorReport) -> String {
    let mut output = String::new();
    output.push_str("# Doctor report\n\n");
    output.push_str(&format!("- schema: {}\n", report.schema));
    output.push_str(&format!("- run id: {}\n", report.run_id));
    output.push_str(&format!("- timestamp: {}\n", report.timestamp));
    output.push_str(&format!(
        "- commit: {}\n",
        report.commit.as_deref().unwrap_or("UNKNOWN")
    ));
    output.push_str(&format!("- OS: {} {}\n", report.os.name, report.os.version));
    output.push_str(&format!(
        "- CPU: {} ({} cores / {} threads)\n",
        report.cpu.model, report.cpu.physical_cores, report.cpu.logical_threads
    ));
    output.push_str(&format!(
        "- memory: {}\n",
        report
            .memory
            .total_bytes
            .map(|bytes| bytes.to_string())
            .unwrap_or_else(|| "UNKNOWN".to_string())
    ));
    output.push_str(&format!(
        "- Rust: {}\n",
        report.rust.version.as_deref().unwrap_or("UNKNOWN")
    ));
    output.push_str(&format!("- GPU available: {}\n", report.gpu.available));
    for adapter in &report.gpu.adapters {
        output.push_str(&format!(
            "  - {} ({}, {}, driver {})\n",
            adapter.name, adapter.backend, adapter.device_type, adapter.driver
        ));
    }
    output.push_str(&format!(
        "- release profile matches PERF-A10: {}\n",
        report.release_profile.matches_perf_a10
    ));
    output.push_str(&format!(
        "- private root: {} (present: {})\n",
        report.private_root.path, report.private_root.present
    ));
    output.push_str(&format!("- launcher found: {}\n", report.launcher.found));
    for install in &report.launcher.installs {
        output.push_str(&format!("  - {install}\n"));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_match_runner_contract() {
        assert_eq!(exit_code(&Outcome::Pass), 0);
        assert_eq!(exit_code(&Outcome::Fail("failure".to_string())), 1);
        assert_eq!(exit_code(&Outcome::Blocked("blocked".to_string())), 2);
    }

    #[test]
    fn report_serializes_schema_version_one() {
        let report = sample_report();
        let value = serde_json::to_value(&report).expect("serializable");
        assert_eq!(value["schema"].as_u64(), Some(1));
        assert_eq!(value["run_id"].as_str(), Some("42"));
        assert_eq!(value["os"]["name"].as_str(), Some("linux-x86_64"));
        assert_eq!(
            value["release_profile"]["matches_perf_a10"].as_bool(),
            Some(true)
        );
    }

    fn sample_report() -> DoctorReport {
        DoctorReport {
            schema: 1,
            run_id: "42".to_string(),
            timestamp: 0,
            commit: None,
            os: OsInfo {
                name: "linux-x86_64".to_string(),
                version: "test".to_string(),
            },
            cpu: CpuInfo {
                model: "test".to_string(),
                physical_cores: 1,
                logical_threads: 1,
            },
            memory: MemoryInfo {
                total_bytes: Some(1024),
            },
            rust: RustInfo {
                version: Some("rustc 1.99.1".to_string()),
            },
            gpu: GpuInfo {
                available: false,
                adapters: Vec::new(),
            },
            release_profile: ReleaseProfile {
                opt_level: Some("3".to_string()),
                lto: Some("fat".to_string()),
                codegen_units: Some("1".to_string()),
                panic: Some("abort".to_string()),
                debug: Some("line-tables-only".to_string()),
                matches_perf_a10: true,
            },
            private_root: PrivateRoot {
                path: "/tmp".to_string(),
                present: false,
            },
            launcher: Launcher {
                found: false,
                installs: Vec::new(),
            },
        }
    }
}

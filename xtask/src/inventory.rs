use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

pub fn run() -> ExitCode {
    match inventory() {
        Ok(()) => {
            println!("inventory: recorded");
            ExitCode::from(0)
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn inventory() -> Result<(), String> {
    let java = find_java().ok_or("Java not found (system java or Prism java-runtime-epsilon)")?;
    let server_jar = find_server_jar().ok_or_else(|| {
        format!(
            "server jar not found under {}; download the 26.3 server jar there",
            private_root().display()
        )
    })?;
    let output_dir = private_root().join("inventory");
    fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    run_data_generator(&java, &server_jar, &output_dir)?;
    let counts = parse_reports(&output_dir)?;
    write_counts(&counts, &output_dir)?;
    Ok(())
}

fn find_java() -> Option<PathBuf> {
    if Command::new("java").arg("-version").output().is_ok() {
        return Some(PathBuf::from("java"));
    }
    let prism = prism_data_dir()?;
    for entry in fs::read_dir(prism.join("java")).ok()?.flatten() {
        let candidate = entry.path().join("bin/java");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn find_server_jar() -> Option<PathBuf> {
    let root = private_root();
    ["server.jar", "minecraft_server.26.3.jar"]
        .iter()
        .map(|name| root.join(name))
        .find(|path| path.is_file())
}

fn run_data_generator(java: &Path, server_jar: &Path, output_dir: &Path) -> Result<(), String> {
    let mut child = Command::new(java)
        .args([
            "-DbundlerMainClass=net.minecraft.data.Main",
            "-jar",
            server_jar.to_str().unwrap_or(""),
            "--reports",
            "--output",
            output_dir.to_str().unwrap_or(""),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to launch data generator: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs(300);
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            if status.success() {
                return Ok(());
            }
            return Err(format!("data generator exited with {status}"));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("data generator timed out".to_string());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn parse_reports(output_dir: &Path) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for entry in fs::read_dir(output_dir)
        .map_err(|error| error.to_string())?
        .flatten()
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let file_counts = parse_registry_counts(&text)?;
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("unknown")
            .to_string();
        counts.insert(name, file_counts.values().sum());
    }
    Ok(counts)
}

fn parse_registry_counts(text: &str) -> Result<BTreeMap<String, usize>, String> {
    let value: Value = serde_json::from_str(text).map_err(|error| error.to_string())?;
    let mut counts = BTreeMap::new();
    if let Value::Object(registries) = value {
        for (name, registry) in registries {
            let count = registry
                .get("entries")
                .and_then(|entries| entries.as_object())
                .map(|entries| entries.len())
                .unwrap_or(0);
            counts.insert(name, count);
        }
    }
    Ok(counts)
}

fn write_counts(counts: &BTreeMap<String, usize>, output_dir: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(counts).map_err(|error| error.to_string())?;
    fs::write(output_dir.join("counts.json"), json).map_err(|error| error.to_string())?;
    Ok(())
}

fn prism_data_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok().map(PathBuf::from)?;
    [
        home.join(".var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher"),
        home.join(".local/share/PrismLauncher"),
    ]
    .into_iter()
    .find(|path| path.is_dir())
}

fn private_root() -> PathBuf {
    let workspace = crate::workspace_root();
    std::env::var("SUNSET_PRIVATE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            workspace
                .parent()
                .unwrap_or(workspace)
                .join("sunset-private")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_registry_counts() {
        let json = r#"{"minecraft:block":{"entries":{"minecraft:air":{},"minecraft:stone":{}}},"minecraft:item":{"entries":{"minecraft:apple":{}}}}"#;
        let counts = parse_registry_counts(json).expect("parses");
        assert_eq!(counts["minecraft:block"], 2);
        assert_eq!(counts["minecraft:item"], 1);
    }

    #[test]
    fn empty_registry_counts_to_zero() {
        let json = r#"{"minecraft:block":{"entries":{}}}"#;
        let counts = parse_registry_counts(json).expect("parses");
        assert_eq!(counts["minecraft:block"], 0);
    }
}

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Serialize)]
struct TargetLock {
    schema: u32,
    edition: String,
    version: String,
    client_jar_sha256: Option<String>,
    server_jar_sha256: Option<String>,
    asset_index_id: Option<String>,
    asset_index_sha256: Option<String>,
    protocol_version: Option<u32>,
    resource_pack_format: Option<u32>,
    data_pack_format: Option<u32>,
    java_runtime: Option<String>,
    launch_flags: Vec<String>,
    oracle_adapter_version: String,
    provenance: Provenance,
}

#[derive(Serialize)]
struct Provenance {
    source: String,
    recorded_at: String,
    tool: String,
}

#[derive(Serialize)]
struct TargetLockPublic {
    schema: u32,
    edition: String,
    version: String,
    asset_index_id: Option<String>,
    protocol_version: Option<u32>,
    resource_pack_format: Option<u32>,
    data_pack_format: Option<u32>,
}

struct JarVersion {
    protocol_version: Option<u32>,
    resource_major: Option<u32>,
    data_major: Option<u32>,
    java_runtime: Option<String>,
}

pub fn run() -> ExitCode {
    match lock() {
        Ok(()) => {
            println!("lock: reference locked");
            ExitCode::from(0)
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn lock() -> Result<(), String> {
    let prism = prism_data_dir().ok_or("Prism Launcher data directory not found")?;
    let client_jar = prism.join("libraries/com/mojang/minecraft/26.3/minecraft-26.3-client.jar");
    if !client_jar.is_file() {
        return Err(format!("client jar not found: {}", client_jar.display()));
    }
    let index_dir = prism.join("assets/indexes");
    let asset_index_id = find_asset_index_id(&index_dir)
        .ok_or_else(|| format!("asset index not found under {}", index_dir.display()))?;
    let asset_index = index_dir.join(format!("{asset_index_id}.json"));
    if !asset_index.is_file() {
        return Err(format!(
            "asset index file not found: {}",
            asset_index.display()
        ));
    }
    let jar_version = read_jar_version(&client_jar)?;
    let lock = TargetLock {
        schema: 1,
        edition: "java".to_string(),
        version: "26.3".to_string(),
        client_jar_sha256: Some(sha256_file(&client_jar)?),
        server_jar_sha256: None,
        asset_index_id: Some(asset_index_id),
        asset_index_sha256: Some(sha256_file(&asset_index)?),
        protocol_version: jar_version.protocol_version,
        resource_pack_format: jar_version.resource_major,
        data_pack_format: jar_version.data_major,
        java_runtime: jar_version.java_runtime,
        launch_flags: Vec::new(),
        oracle_adapter_version: "NONE".to_string(),
        provenance: Provenance {
            source: "local launcher install".to_string(),
            recorded_at: timestamp().to_string(),
            tool: format!("xtask lock {}", env!("CARGO_PKG_VERSION")),
        },
    };
    let public = TargetLockPublic {
        schema: lock.schema,
        edition: lock.edition.clone(),
        version: lock.version.clone(),
        asset_index_id: lock.asset_index_id.clone(),
        protocol_version: lock.protocol_version,
        resource_pack_format: lock.resource_pack_format,
        data_pack_format: lock.data_pack_format,
    };
    write_private(&lock)?;
    write_public(&public)?;
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

fn find_asset_index_id(index_dir: &Path) -> Option<String> {
    let mut ids = Vec::new();
    for entry in fs::read_dir(index_dir).ok()?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                ids.push(stem.to_string());
            }
        }
    }
    ids.sort();
    ids.into_iter().next()
}

fn read_jar_version(jar: &Path) -> Result<JarVersion, String> {
    let jar_path = jar.to_str().ok_or("jar path is not valid UTF-8")?;
    let output = Command::new("unzip")
        .args(["-p", jar_path, "version.json"])
        .output()
        .map_err(|error| format!("unzip failed: {error}"))?;
    if !output.status.success() {
        return Err("unzip could not read version.json from the client jar".to_string());
    }
    parse_jar_version(&String::from_utf8_lossy(&output.stdout))
}

fn parse_jar_version(text: &str) -> Result<JarVersion, String> {
    let value: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let protocol_version = value
        .get("protocol_version")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let pack = value.get("pack_version");
    let resource_major = pack
        .and_then(|p| p.get("resource_major"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let data_major = pack
        .and_then(|p| p.get("data_major"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let component = value
        .get("java_component")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let version = value.get("java_version").and_then(|v| v.as_u64());
    let java_runtime = match (component, version) {
        (Some(component), Some(version)) => Some(format!("{component} {version}")),
        (Some(component), None) => Some(component),
        _ => None,
    };
    Ok(JarVersion {
        protocol_version,
        resource_major,
        data_major,
        java_runtime,
    })
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(hex_sha256(&bytes))
}

pub(crate) fn hex_sha256(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest.iter() {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
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

fn write_private(lock: &TargetLock) -> Result<(), String> {
    let root = private_root();
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let json = serde_json::to_string_pretty(lock).map_err(|error| error.to_string())?;
    fs::write(root.join("target-lock.json"), json).map_err(|error| error.to_string())?;
    Ok(())
}

fn write_public(lock: &TargetLockPublic) -> Result<(), String> {
    let json = serde_json::to_string_pretty(lock).map_err(|error| error.to_string())?;
    fs::write(
        crate::workspace_root().join("target-lock.public.json"),
        json,
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_of_known_bytes() {
        assert_eq!(
            hex_sha256(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn parse_version_json_extracts_protocol_and_packs() {
        let json = r#"{"protocol_version":777,"pack_version":{"resource_major":97,"data_major":121},"java_version":25,"java_component":"java-runtime-epsilon"}"#;
        let parsed = parse_jar_version(json).expect("parses");
        assert_eq!(parsed.protocol_version, Some(777));
        assert_eq!(parsed.resource_major, Some(97));
        assert_eq!(parsed.data_major, Some(121));
        assert_eq!(
            parsed.java_runtime.as_deref(),
            Some("java-runtime-epsilon 25")
        );
    }
}

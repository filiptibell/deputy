use std::{collections::HashMap, path::Path, process::Stdio};

use serde::Deserialize;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tracing::debug;

use deputy_versioning::Version;

use super::CratesClient;
use super::models::LocalMetadata;

impl CratesClient {
    pub async fn get_local_metadata(&self, manifest_dir: &Path) -> Option<LocalMetadata> {
        let cache_key = manifest_dir.to_string_lossy().to_string();

        let manifest_dir = manifest_dir.to_path_buf();
        let fut = async {
            debug!(
                "Fetching local crate metadata for '{}'",
                manifest_dir.display()
            );

            if let Some(meta) = try_cargo_metadata(&manifest_dir).await {
                return Some(meta);
            }

            parse_local_manifest(&manifest_dir.join("Cargo.toml")).await
        };

        self.cache
            .local_metadatas
            .with_caching(cache_key, fut)
            .await
    }
}

// The full and proper `cargo metadata` output is our primary source...

#[derive(Deserialize)]
struct CargoMetadataOutput {
    packages: Vec<CargoMetadataPackage>,
}

#[derive(Deserialize)]
struct CargoMetadataPackage {
    manifest_path: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    features: HashMap<String, Vec<String>>,
}

async fn try_cargo_metadata(manifest_dir: &Path) -> Option<LocalMetadata> {
    let manifest_path = manifest_dir.join("Cargo.toml");

    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let meta: CargoMetadataOutput = serde_json::from_slice(&output.stdout).ok()?;

    let manifest_path_str = manifest_path.to_string_lossy();
    let package = meta
        .packages
        .into_iter()
        .find(|p| p.manifest_path == manifest_path_str.as_ref())?;

    let version = package.version.and_then(|v| v.parse::<Version>().ok());

    let features = package.features.into_keys().collect();

    Some(LocalMetadata { version, features })
}

// ... and we fall back to a primitive but fast file reader, if cargo is not available

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManifestSection {
    Other,
    Package,
    Features,
}

async fn parse_local_manifest(path: &Path) -> Option<LocalMetadata> {
    let file = File::open(path).await.ok()?;
    let reader = BufReader::new(file);

    let mut version = None;
    let mut features = Vec::new();
    let mut section = ManifestSection::Other;

    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            section = match trimmed {
                "[package]" => ManifestSection::Package,
                "[features]" => ManifestSection::Features,
                _ => ManifestSection::Other,
            };
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();

        match section {
            ManifestSection::Package if key == "version" => {
                let value = value.trim().trim_matches('"');
                version = value.parse().ok();
            }
            ManifestSection::Features => {
                features.push(key.to_string());
            }
            _ => {}
        }
    }

    Some(LocalMetadata { version, features })
}

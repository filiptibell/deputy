use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
};

use serde::Deserialize;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tracing::debug;

use deputy_versioning::Version;

use super::CratesClient;
use super::models::{LocalMetadata, WorkspaceDependencyMetadata, WorkspacePackageMetadata};

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

    pub async fn get_workspace_package_metadata(
        &self,
        manifest_path: &Path,
    ) -> Option<WorkspacePackageMetadata> {
        let cache_key = manifest_path.to_string_lossy().to_string();

        let manifest_path = manifest_path.to_path_buf();
        let fut = async {
            debug!(
                "Fetching workspace package metadata for '{}'",
                manifest_path.display()
            );

            try_workspace_package_metadata(&manifest_path).await
        };

        self.cache
            .workspace_package_metadatas
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
    dependencies: Vec<CargoMetadataDependency>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    features: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct CargoMetadataDependency {
    name: String,
    req: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    rename: Option<String>,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    path: Option<PathBuf>,
}

async fn try_cargo_metadata(manifest_dir: &Path) -> Option<LocalMetadata> {
    let manifest_path = manifest_dir.join("Cargo.toml");
    let meta = run_cargo_metadata(&manifest_path).await?;

    let package = meta
        .packages
        .into_iter()
        .find(|p| manifest_matches(&p.manifest_path, &manifest_path))?;

    let version = package.version.and_then(|v| v.parse::<Version>().ok());

    let features = package.features.into_keys().collect();

    Some(LocalMetadata { version, features })
}

async fn try_workspace_package_metadata(manifest_path: &Path) -> Option<WorkspacePackageMetadata> {
    let meta = run_cargo_metadata(manifest_path).await?;

    let package = meta
        .packages
        .into_iter()
        .find(|p| manifest_matches(&p.manifest_path, manifest_path))?;

    Some(WorkspacePackageMetadata {
        dependencies: package.dependencies.into_iter().map(Into::into).collect(),
    })
}

async fn run_cargo_metadata(manifest_path: &Path) -> Option<CargoMetadataOutput> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .arg("--no-deps")
        .arg("--manifest-path")
        .arg(manifest_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    serde_json::from_slice(&output.stdout).ok()
}

fn manifest_matches(left: &str, right: &Path) -> bool {
    Path::new(left) == right
}

impl From<CargoMetadataDependency> for WorkspaceDependencyMetadata {
    fn from(dep: CargoMetadataDependency) -> Self {
        Self {
            manifest_name: dep.rename.unwrap_or_else(|| dep.name.clone()),
            name: dep.name,
            req: dep.req,
            source: dep.source,
            features: dep.features,
            path: dep.path,
        }
    }
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

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use tokio::runtime::Builder;

    use super::*;

    #[test]
    fn gets_workspace_package_metadata() {
        let root = tempdir().expect("temp workspace can be created");
        let member = root.path().join("member");
        let member_src = member.join("src");

        fs::create_dir_all(&member_src).expect("temp workspace can be created");
        fs::write(
            root.path().join("Cargo.toml"),
            r#"
[workspace]
members = ["member"]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
"#,
        )
        .expect("workspace manifest can be written");
        fs::write(
            member.join("Cargo.toml"),
            r#"
[package]
name = "member"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true, features = ["rc"] }
"#,
        )
        .expect("member manifest can be written");
        fs::write(member_src.join("lib.rs"), "").expect("member lib can be written");

        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime can be created");
        let metadata = runtime
            .block_on(
                CratesClient::new().get_workspace_package_metadata(&member.join("Cargo.toml")),
            )
            .expect("workspace package metadata exists");

        let dep = metadata
            .dependency("serde")
            .expect("workspace dependency is resolved");
        assert_eq!(dep.name, "serde");
        assert_eq!(dep.manifest_name, "serde");
        assert_eq!(dep.req, "^1.0");
        assert_eq!(dep.features, vec!["derive", "rc"]);
        assert!(
            dep.source
                .as_ref()
                .is_some_and(|s| s.starts_with("registry+"))
        );
    }
}

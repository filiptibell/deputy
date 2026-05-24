use std::{
    fs,
    path::{Path, PathBuf},
};

use async_language_server::{lsp_types::Url, server::Document};

use deputy_clients::crates::models::{LocalMetadata, WorkspaceDependencyMetadata};
use deputy_parser::cargo::{self, CargoDependency};
use deputy_versioning::{VersionReq, Versioned};

use super::Clients;

pub async fn get_features(clients: &Clients, dname: &str, dver: &str) -> Option<Vec<String>> {
    let dreq = VersionReq::parse(dver).ok()?;

    let metas = clients
        .crates
        .get_sparse_index_crate_metadatas(dname)
        .await
        .inspect_err(|e| {
            tracing::error!("failed to get crate data for {dname}: {e}");
        })
        .ok()?;

    let meta = metas.iter().find_map(|meta| {
        let version = meta.parse_version().ok()?;
        if dreq.matches(&version) {
            Some(meta)
        } else {
            None
        }
    })?;

    Some(
        meta.all_features()
            .into_iter()
            .map(ToString::to_string)
            .collect(),
    )
}

pub async fn get_local_metadata(
    clients: &Clients,
    doc_url: &Url,
    relative_path: &str,
) -> Option<LocalMetadata> {
    let LocalDependencyPathResolution::Resolved { manifest_dir } =
        resolve_local_dependency_path(doc_url, relative_path)?
    else {
        return None;
    };

    clients.crates.get_local_metadata(&manifest_dir).await
}

#[derive(Debug, Clone)]
pub enum LocalDependencyResolution {
    Resolved(LocalMetadata),
    MissingPath,
    MissingManifest,
    Unavailable,
}

pub async fn resolve_local_dependency(
    clients: &Clients,
    doc_url: &Url,
    relative_path: &str,
) -> LocalDependencyResolution {
    match resolve_local_dependency_path(doc_url, relative_path) {
        Some(LocalDependencyPathResolution::Resolved { manifest_dir }) => clients
            .crates
            .get_local_metadata(&manifest_dir)
            .await
            .map_or(LocalDependencyResolution::Unavailable, |metadata| {
                LocalDependencyResolution::Resolved(metadata)
            }),
        Some(LocalDependencyPathResolution::MissingPath) => LocalDependencyResolution::MissingPath,
        Some(LocalDependencyPathResolution::MissingManifest) => {
            LocalDependencyResolution::MissingManifest
        }
        None => LocalDependencyResolution::Unavailable,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalDependencyPathResolution {
    Resolved { manifest_dir: PathBuf },
    MissingPath,
    MissingManifest,
}

fn resolve_local_dependency_path(
    doc_url: &Url,
    relative_path: &str,
) -> Option<LocalDependencyPathResolution> {
    let doc_path = doc_url.to_file_path().ok()?;
    let doc_dir = doc_path.parent()?;

    let dep_path = doc_dir.join(relative_path);
    if dep_path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("toml"))
    {
        if dep_path.is_file() {
            let manifest_dir = dep_path.parent()?.canonicalize().ok()?;
            return Some(LocalDependencyPathResolution::Resolved { manifest_dir });
        }
        return Some(LocalDependencyPathResolution::MissingManifest);
    }

    if !dep_path.exists() {
        return Some(LocalDependencyPathResolution::MissingPath);
    }

    let manifest_path = dep_path.join("Cargo.toml");
    if !manifest_path.is_file() {
        return Some(LocalDependencyPathResolution::MissingManifest);
    }

    Some(LocalDependencyPathResolution::Resolved {
        manifest_dir: dep_path.canonicalize().ok()?,
    })
}

#[derive(Debug, Clone)]
pub enum WorkspaceDependencyResolution {
    NotWorkspace,
    Resolved(WorkspaceDependencyMetadata),
    Missing { name: String },
    Unavailable,
}

pub async fn resolve_workspace_dependency(
    clients: &Clients,
    doc: &Document,
    dep: &CargoDependency<'_>,
) -> WorkspaceDependencyResolution {
    if !dep.is_workspace() {
        return WorkspaceDependencyResolution::NotWorkspace;
    }

    let Some(manifest_path) = doc.url().to_file_path().ok() else {
        return WorkspaceDependencyResolution::Unavailable;
    };
    let (name, _) = dep.text(doc);

    if let Some(metadata) = clients
        .crates
        .get_workspace_package_metadata(&manifest_path)
        .await
        .and_then(|metadata| metadata.dependency(&name).cloned())
    {
        return WorkspaceDependencyResolution::Resolved(metadata);
    }

    match workspace_dependency_exists(&manifest_path, &name) {
        Some(false) => WorkspaceDependencyResolution::Missing { name },
        Some(true) | None => WorkspaceDependencyResolution::Unavailable,
    }
}

fn workspace_dependency_exists(manifest_path: &Path, name: &str) -> Option<bool> {
    let manifest_path = find_workspace_manifest_path(manifest_path)?;
    let text = fs::read_to_string(manifest_path).ok()?;
    Some(
        cargo::workspace_dependency_names_from_text(&text)
            .into_iter()
            .any(|dep| dep == name),
    )
}

fn find_workspace_manifest_path(manifest_path: &Path) -> Option<PathBuf> {
    let mut dir = manifest_path.parent();
    while let Some(current_dir) = dir {
        let candidate = current_dir.join("Cargo.toml");
        if fs::read_to_string(&candidate)
            .ok()
            .is_some_and(|text| cargo::manifest_has_workspace(&text))
        {
            return Some(candidate);
        }
        dir = current_dir.parent();
    }
    None
}

pub async fn get_workspace_local_metadata(
    clients: &Clients,
    metadata: &WorkspaceDependencyMetadata,
) -> Option<LocalMetadata> {
    let path = metadata.path.as_deref()?;
    clients.crates.get_local_metadata(path).await
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn checks_workspace_dependency_existence_from_disk() {
        let root = tempdir().expect("temp workspace can be created");
        let member = root.path().join("member");

        fs::create_dir_all(&member).expect("temp workspace can be created");
        fs::write(
            root.path().join("Cargo.toml"),
            r#"
[workspace]
members = ["member"]
"#,
        )
        .expect("workspace manifest can be written");
        let manifest_path = member.join("Cargo.toml");
        fs::write(&manifest_path, "").expect("member manifest can be written");

        assert_eq!(
            workspace_dependency_exists(&manifest_path, "serde"),
            Some(false)
        );

        fs::write(
            root.path().join("Cargo.toml"),
            r#"
[workspace]
members = ["member"]

[workspace.dependencies]
serde = "1.0"
"#,
        )
        .expect("workspace manifest can be updated");

        assert_eq!(
            workspace_dependency_exists(&manifest_path, "serde"),
            Some(true)
        );
    }

    #[test]
    fn checks_dotted_workspace_dependency_existence_from_disk() {
        let root = tempdir().expect("temp workspace can be created");
        let member = root.path().join("member");

        fs::create_dir_all(&member).expect("temp workspace can be created");
        fs::write(
            root.path().join("Cargo.toml"),
            r#"
[workspace]
members = ["member"]

dependencies.foo.version = "*"
dependencies.foo.package = "rand"
"#,
        )
        .expect("workspace manifest can be written");
        let manifest_path = member.join("Cargo.toml");
        fs::write(&manifest_path, "").expect("member manifest can be written");

        assert_eq!(
            workspace_dependency_exists(&manifest_path, "foo"),
            Some(true)
        );
        assert_eq!(
            workspace_dependency_exists(&manifest_path, "rand"),
            Some(false)
        );
    }

    #[test]
    fn resolves_local_dependency_paths_from_disk() {
        let root = tempdir().expect("temp workspace can be created");
        let member = root.path().join("member");
        let dep = root.path().join("dep");

        fs::create_dir_all(&member).expect("temp member can be created");
        let manifest_path = member.join("Cargo.toml");
        fs::write(&manifest_path, "").expect("member manifest can be written");
        let manifest_url = Url::from_file_path(&manifest_path).expect("manifest path is absolute");

        assert_eq!(
            resolve_local_dependency_path(&manifest_url, "../dep"),
            Some(LocalDependencyPathResolution::MissingPath)
        );

        fs::create_dir_all(&dep).expect("temp dependency can be created");
        assert_eq!(
            resolve_local_dependency_path(&manifest_url, "../dep"),
            Some(LocalDependencyPathResolution::MissingManifest)
        );

        fs::write(dep.join("Cargo.toml"), "").expect("dependency manifest can be written");
        let dep = dep
            .canonicalize()
            .expect("dependency path can be canonicalized");
        assert_eq!(
            resolve_local_dependency_path(&manifest_url, "../dep"),
            Some(LocalDependencyPathResolution::Resolved { manifest_dir: dep })
        );
    }
}

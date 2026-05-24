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
    let doc_path = doc_url.to_file_path().ok()?;
    let doc_dir = doc_path.parent()?;

    let dep_path = doc_dir.join(relative_path);
    let manifest_dir = if dep_path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("toml"))
    {
        dep_path.parent()?
    } else {
        dep_path.as_path()
    };

    clients.crates.get_local_metadata(manifest_dir).await
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
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    #[test]
    fn checks_workspace_dependency_existence_from_disk() {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after unix epoch")
            .as_millis();
        let root = std::env::temp_dir().join(format!("deputy-workspace-deps-{millis}"));
        let member = root.join("member");

        fs::create_dir_all(&member).expect("temp workspace can be created");
        fs::write(
            root.join("Cargo.toml"),
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
            root.join("Cargo.toml"),
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

        fs::remove_dir_all(root).expect("temp workspace can be removed");
    }
}

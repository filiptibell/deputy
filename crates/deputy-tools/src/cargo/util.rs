use async_language_server::{lsp_types::Url, server::Document};

use deputy_clients::crates::models::{LocalMetadata, WorkspaceDependencyMetadata};
use deputy_parser::cargo::CargoDependency;
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

pub async fn get_workspace_dependency_metadata(
    clients: &Clients,
    doc: &Document,
    dep: &CargoDependency<'_>,
) -> Option<WorkspaceDependencyMetadata> {
    if !dep.is_workspace() {
        return None;
    }

    let manifest_path = doc.url().to_file_path().ok()?;
    let (name, _) = dep.text(doc);
    clients
        .crates
        .get_workspace_package_metadata(&manifest_path)
        .await?
        .dependency(&name)
        .cloned()
}

pub async fn get_workspace_local_metadata(
    clients: &Clients,
    metadata: &WorkspaceDependencyMetadata,
) -> Option<LocalMetadata> {
    let path = metadata.path.as_deref()?;
    clients.crates.get_local_metadata(path).await
}

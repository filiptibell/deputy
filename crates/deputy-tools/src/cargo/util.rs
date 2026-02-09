use async_language_server::lsp_types::Url;

use deputy_clients::crates::models::LocalMetadata;
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

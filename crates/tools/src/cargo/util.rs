use semver::VersionReq;

use shared::Versioned;

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

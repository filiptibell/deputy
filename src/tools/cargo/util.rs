use semver::VersionReq;

use crate::{parser::Dependency, util::Versioned};

use super::Clients;

pub async fn get_features(clients: &Clients, dep: &Dependency) -> Option<Vec<String>> {
    let dname = dep.name().unquoted();
    let dver = dep.spec().and_then(|s| s.contents.version.as_ref())?;
    let dreq = VersionReq::parse(dver.unquoted()).ok()?;

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
            .map(|s| s.to_string())
            .collect(),
    )
}

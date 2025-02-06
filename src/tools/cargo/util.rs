use semver::VersionReq;

use crate::{parser::Dependency, util::Versioned};

use super::Clients;

pub async fn get_features(clients: &Clients, dep: &Dependency) -> Option<Vec<String>> {
    let dname = dep.name().unquoted();
    let dver = dep.spec().and_then(|s| s.contents.version.as_ref())?;
    let dreq = VersionReq::parse(dver.unquoted()).ok()?;

    let data = clients
        .crates
        .get_crate_data(dname)
        .await
        .inspect_err(|e| {
            tracing::error!("failed to get crate data for {dname}: {e}");
        })
        .ok()?;

    let features = data.versions.into_iter().find_map(|meta| {
        let version = meta.parse_version().ok()?;
        if dreq.matches(&version) {
            Some(meta.features)
        } else {
            None
        }
    })?;

    let mut known_features = features.into_keys().collect::<Vec<_>>();
    known_features.sort_unstable();
    known_features.dedup();
    Some(known_features)
}

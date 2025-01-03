use crate::parser::Dependency;

use super::Clients;

pub async fn get_features(clients: &Clients, dep: &Dependency) -> Vec<String> {
    let dname = dep.name().unquoted();

    let Ok(metas) = clients.crates.get_sparse_index_crate_metadatas(dname).await else {
        return Vec::new();
    };

    let mut known_features = metas
        .iter()
        .flat_map(|meta| meta.features.keys().cloned())
        .collect::<Vec<_>>();

    known_features.sort_unstable();
    known_features.dedup();
    known_features
}

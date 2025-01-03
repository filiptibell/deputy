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

pub fn did_you_mean<S1, S2>(current: S1, options: &[S2]) -> Option<String>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let current = current.as_ref();
    let options = options.iter().map(|s| s.as_ref()).collect::<Vec<_>>();

    let (best_score, best_option) = options
        .iter()
        .map(|s| {
            let score = strsim::jaro_winkler(current, s);
            ((score * (u64::MAX as f64)) as u64, s)
        })
        .max_by_key(|(score, _)| *score)?;

    if (best_score as f64) / (u64::MAX as f64) >= 0.65 {
        Some(best_option.to_string())
    } else {
        None
    }
}

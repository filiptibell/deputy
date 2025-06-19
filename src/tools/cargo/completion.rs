use tracing::debug;

use async_language_server::{
    lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse},
    server::{Document, ServerResult},
};

use crate::clients::*;
use crate::parser::{Dependency, Node};
use crate::tools::cargo::constants::CratesIoPackage;
use crate::tools::cargo::util::get_features;

use super::constants::top_crates_io_packages_prefixed;
use super::Versioned;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;
const MINIMUM_PACKAGES_BEFORE_FETCH: usize = 16; // Less than 16 packages found statically = fetch dynamically

pub async fn get_cargo_completions_name(
    clients: &Clients,
    _document: &Document,
    dep: &Dependency,
) -> ServerResult<Option<CompletionResponse>> {
    let dname = dep.name().unquoted();

    let mut packages = top_crates_io_packages_prefixed(dname, MAXIMUM_PACKAGES_SHOWN)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    if packages.len() < MINIMUM_PACKAGES_BEFORE_FETCH {
        if let Ok(crates) = clients.crates.search_crates(dname).await {
            let count_prev = packages.len();

            packages.extend(crates.inner.into_iter().map(|m| CratesIoPackage {
                name: m.name.to_string().into(),
                downloads: m.downloads.total_count,
                description: m.description.to_string().into(),
            }));

            packages.sort_by_key(|package| package.name.to_ascii_lowercase());
            packages.dedup_by_key(|p| p.name.to_ascii_lowercase());
            packages.truncate(MINIMUM_PACKAGES_BEFORE_FETCH);

            let count_after = packages.len();
            if count_after > count_prev {
                debug!(
                    "Found {} additional crates for prefix '{dname}'",
                    count_after.saturating_sub(count_prev),
                );
            }
        }
    }

    let items = packages
        .into_iter()
        .map(|package| CompletionItem {
            label: package.name.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            detail: Some(package.description.to_string()),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

pub async fn get_cargo_completions_version(
    clients: &Clients,
    _document: &Document,
    dep: &Dependency,
) -> ServerResult<Option<CompletionResponse>> {
    let name = dep.name().unquoted();
    let Some(version) = dep.spec().and_then(|s| s.contents.version.as_ref()) else {
        return Ok(None);
    };

    let metadatas = match clients.crates.get_sparse_index_crate_metadatas(name).await {
        Err(_) => return Ok(None),
        Ok(m) => m,
    };

    let _use_precise_edit = !version.unquoted().is_empty();
    let valid_vec = dep
        .extract_completion_versions(metadatas.into_iter())
        .into_iter()
        .take(MAXIMUM_PACKAGES_SHOWN)
        .enumerate()
        .map(|(index, potential_version)| CompletionItem {
            label: potential_version.item_version_raw.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{:0>5}", index)),
            deprecated: Some(potential_version.item.yanked),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(valid_vec)))
}

pub async fn get_cargo_completions_features(
    clients: &Clients,
    _document: &Document,
    dep: &Dependency,
    feat: &Node<String>,
) -> ServerResult<Option<CompletionResponse>> {
    let Some(known_features) = get_features(clients, dep).await else {
        return Ok(None);
    };

    tracing::debug!("Known features: {known_features:?}");

    let valid_features = known_features
        .into_iter()
        .filter(|f| f.starts_with(feat.unquoted()))
        .enumerate()
        .map(|(index, known_feat)| CompletionItem {
            label: known_feat.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{:0>5}", index)),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(valid_features)))
}

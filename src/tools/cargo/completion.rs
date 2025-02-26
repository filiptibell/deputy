use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tracing::debug;

use crate::clients::*;
use crate::parser::{Dependency, Node};
use crate::server::*;
use crate::tools::cargo::constants::CratesIoPackage;
use crate::tools::cargo::util::get_features;

use super::constants::top_crates_io_packages_prefixed;
use super::Versioned;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;
const MINIMUM_PACKAGES_BEFORE_FETCH: usize = 16; // Less than 16 packages found statically = fetch dynamically

pub async fn get_cargo_completions_name(
    clients: &Clients,
    document: &Document,
    dep: &Dependency,
) -> Result<CompletionResponse> {
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
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(dep.name().range, package.name.to_string()),
            )),
            detail: Some(package.description.to_string()),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(CompletionResponse::Array(items))
}

pub async fn get_cargo_completions_version(
    clients: &Clients,
    document: &Document,
    dep: &Dependency,
) -> Result<CompletionResponse> {
    let name = dep.name().unquoted();
    let Some(version) = dep.spec().and_then(|s| s.contents.version.as_ref()) else {
        return Ok(CompletionResponse::Array(Vec::new()));
    };

    let metadatas = match clients.crates.get_sparse_index_crate_metadatas(name).await {
        Err(_) => return Ok(CompletionResponse::Array(Vec::new())),
        Ok(m) => m,
    };

    let use_precise_edit = !version.unquoted().is_empty();
    let valid_vec = dep
        .extract_completion_versions(metadatas.into_iter())
        .into_iter()
        .take(MAXIMUM_PACKAGES_SHOWN)
        .enumerate()
        .map(|(index, potential_version)| CompletionItem {
            label: potential_version.item_version_raw.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{:0>5}", index)),
            text_edit: if use_precise_edit {
                Some(CompletionTextEdit::Edit(document.create_substring_edit(
                    version.range.start.line,
                    version.unquoted(),
                    potential_version.item_version_raw,
                )))
            } else {
                None
            },
            deprecated: Some(potential_version.item.yanked),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(CompletionResponse::Array(valid_vec))
}

pub async fn get_cargo_completions_features(
    clients: &Clients,
    document: &Document,
    dep: &Dependency,
    feat: &Node<String>,
) -> Result<CompletionResponse> {
    let Some(known_features) = get_features(clients, dep).await else {
        return Ok(CompletionResponse::Array(Vec::new()));
    };

    tracing::debug!("Known features: {known_features:?}");

    let use_precise_edit = !feat.unquoted().is_empty();
    let valid_features = known_features
        .into_iter()
        .filter(|f| f.starts_with(feat.unquoted()))
        .enumerate()
        .map(|(index, known_feat)| CompletionItem {
            label: known_feat.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{:0>5}", index)),
            text_edit: if use_precise_edit {
                Some(CompletionTextEdit::Edit(document.create_substring_edit(
                    feat.range.start.line,
                    feat.unquoted(),
                    known_feat.to_string(),
                )))
            } else {
                None
            },
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(CompletionResponse::Array(valid_features))
}

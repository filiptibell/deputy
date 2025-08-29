use tracing::debug;

use async_language_server::{
    lsp_types::{
        CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit, Position,
        Range, TextEdit,
    },
    server::{Document, ServerResult},
    text_utils::RangeExt,
    tree_sitter::Node,
    tree_sitter_utils::{ts_range_contains_lsp_position, ts_range_to_lsp_range},
};

use deputy_clients::Clients;
use deputy_parser::cargo;
use deputy_parser::utils::unquote;
use deputy_versioning::Versioned;

use crate::cargo::constants::CratesIoPackage;
use crate::cargo::util::get_features;

use super::constants::top_crates_io_packages_prefixed;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;
const MINIMUM_PACKAGES_BEFORE_FETCH: usize = 16; // Less than 16 packages found statically = fetch dynamically

pub async fn get_cargo_completions(
    clients: &Clients,
    doc: &Document,
    pos: Position,
    node: Node<'_>,
) -> ServerResult<Option<CompletionResponse>> {
    let Some(dep) = cargo::parse_dependency(doc, node) else {
        return Ok(None);
    };

    let (name, version) = dep.text(doc);

    // Try to complete names
    if ts_range_contains_lsp_position(dep.name.range(), pos) {
        debug!("Completing name: {dep:?}");
        return complete_name(
            clients,
            name.as_str(),
            ts_range_to_lsp_range(dep.name.range()),
        )
        .await;
    }

    // Try to complete versions
    if ts_range_contains_lsp_position(dep.version.range(), pos) {
        debug!("Completing version: {dep:?}");
        return complete_version(
            clients,
            name.as_str(),
            version.as_str(),
            ts_range_to_lsp_range(dep.version.range()),
        )
        .await;
    }

    // Try to complete features
    for feat_node in dep.feature_nodes() {
        let feat = doc.node_text(feat_node);
        if ts_range_contains_lsp_position(feat_node.range(), pos) {
            debug!("Completing features: {dep:?}");
            return complete_features(
                clients,
                name.as_str(),
                version.as_str(),
                unquote(feat).as_str(),
                ts_range_to_lsp_range(feat_node.range()),
            )
            .await;
        }
    }

    // No completions yet - probably empty dep
    Ok(None)
}

async fn complete_name(
    clients: &Clients,
    name: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let mut packages = top_crates_io_packages_prefixed(name, MAXIMUM_PACKAGES_SHOWN)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    if packages.len() < MINIMUM_PACKAGES_BEFORE_FETCH
        && let Ok(crates) = clients.crates.search_crates(name).await
    {
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
                "Found {} additional crates for prefix '{name}'",
                count_after.saturating_sub(count_prev),
            );
        }
    }

    let items = packages
        .into_iter()
        .map(|package| CompletionItem {
            label: package.name.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            detail: Some(package.description.to_string()),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: package.name.to_string(),
                range,
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

async fn complete_version(
    clients: &Clients,
    name: &str,
    version: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let Ok(metadatas) = clients.crates.get_sparse_index_crate_metadatas(name).await else {
        return Ok(None);
    };

    let valid_vec = version
        .extract_completion_versions(metadatas.into_iter())
        .into_iter()
        .take(MAXIMUM_PACKAGES_SHOWN)
        .enumerate()
        .map(|(index, potential_version)| CompletionItem {
            label: potential_version.item_version_raw.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{index:0>5}")),
            deprecated: Some(potential_version.item.yanked),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: potential_version.item_version_raw.to_string(),
                range: range.shrink(1, 1),
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(valid_vec)))
}

async fn complete_features(
    clients: &Clients,
    name: &str,
    version: &str,
    feat: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let Some(known_features) = get_features(clients, name, version).await else {
        return Ok(None);
    };

    tracing::debug!("Known features: {known_features:?}");

    let valid_features = known_features
        .into_iter()
        .filter(|f| f.starts_with(feat))
        .enumerate()
        .map(|(index, known_feat)| CompletionItem {
            label: known_feat.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{index:0>5}")),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: known_feat.to_string(),
                range: range.shrink(1, 1),
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(valid_features)))
}

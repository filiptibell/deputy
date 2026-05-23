use async_language_server::{
    lsp_types::{
        CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit, Position,
        Range, TextEdit,
    },
    server::{Document, ServerResult},
    tree_sitter::Node,
    tree_sitter_utils::{ts_range_contains_lsp_position, ts_range_to_lsp_range},
};
use tracing::debug;

use deputy_parser::gomod;
use deputy_versioning::Versioned;

use crate::shared::filter_starts_with;

use super::Clients;
use super::constants::{GoPackage, top_go_packages_prefixed};

const MAXIMUM_PACKAGES_SHOWN: usize = 64;
const MAXIMUM_VERSIONS_SHOWN: usize = 64;
const MINIMUM_PACKAGES_BEFORE_FETCH: usize = 16; // Less than 16 packages found statically = fetch dynamically
const MINIMUM_QUERY_LENGTH_BEFORE_FETCH: usize = 4; // Avoid broad pkg.go.dev searches for very short prefixes

pub async fn get_gomod_completions(
    clients: &Clients,
    doc: &Document,
    pos: Position,
    node: Node<'_>,
) -> ServerResult<Option<CompletionResponse>> {
    let Some(dep) = gomod::parse_dependency(node) else {
        return Ok(None);
    };

    let (path, version) = dep.text(doc);

    // Try to complete versions
    if let Some(version_node) = dep.version
        && ts_range_contains_lsp_position(version_node.range(), pos)
    {
        debug!("Completing version: {dep:?}");
        return complete_version(
            clients,
            &path,
            version.as_deref().unwrap_or_default(),
            ts_range_to_lsp_range(version_node.range()),
        )
        .await;
    }

    // Try to complete module paths
    if ts_range_contains_lsp_position(dep.path.range(), pos) {
        debug!("Completing name: {dep:?}");
        return complete_name(clients, &path, ts_range_to_lsp_range(dep.path.range())).await;
    }

    Ok(None)
}

async fn complete_version(
    clients: &Clients,
    module_path: &str,
    version: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let Ok(versions) = clients.golang.get_module_versions(module_path).await else {
        return Ok(None);
    };

    // Strip v prefix for semver comparison
    let version_trimmed = version.trim_start_matches('v');

    let items = version_trimmed
        .extract_completion_versions(versions.items.into_iter())
        .into_iter()
        .take(MAXIMUM_VERSIONS_SHOWN)
        .enumerate()
        .map(|(index, potential_version)| {
            let display = potential_version.item.version;
            CompletionItem {
                label: display.clone(),
                kind: Some(CompletionItemKind::VALUE),
                sort_text: Some(format!("{index:0>5}")),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    new_text: display,
                    range,
                })),
                ..Default::default()
            }
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(items)))
}

async fn complete_name(
    clients: &Clients,
    path: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let mut packages = top_go_packages_prefixed(path, MAXIMUM_PACKAGES_SHOWN)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    if path.len() >= MINIMUM_QUERY_LENGTH_BEFORE_FETCH
        && packages.len() < MINIMUM_PACKAGES_BEFORE_FETCH
        && let Ok(results) = clients.golang.search(path).await
    {
        let count_prev = packages.len();
        let is_path_query = path.contains('/');

        packages.extend(
            results
                .items
                .into_iter()
                .filter(|package| package.module_path != "std")
                .filter(|package| {
                    !is_path_query || filter_starts_with(package.module_path.as_str(), path)
                })
                .map(|package| GoPackage {
                    path: package.module_path.into(),
                    name: package.package_path.into(),
                    description: package.synopsis.into(),
                }),
        );

        packages.sort_by_key(|package| package.path.to_ascii_lowercase());
        packages.dedup_by_key(|p| p.path.to_ascii_lowercase());
        packages.truncate(MINIMUM_PACKAGES_BEFORE_FETCH);

        let count_after = packages.len();
        if count_after > count_prev {
            debug!(
                "Found {} additional Go modules for prefix '{path}'",
                count_after.saturating_sub(count_prev),
            );
        }
    }

    let items = packages
        .into_iter()
        .map(|package| CompletionItem {
            label: package.path.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            detail: Some(package.description.to_string()).filter(|s| !s.is_empty()),
            filter_text: Some(format!("{} {}", package.path, package.name)),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: package.path.to_string(),
                range,
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(items)))
}

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

use super::Clients;
use super::constants::top_go_packages_prefixed;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;
const MAXIMUM_VERSIONS_SHOWN: usize = 64;

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
        return complete_name(&path, ts_range_to_lsp_range(dep.path.range()));
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

    // Strip v prefix from proxy versions for semver compatibility,
    // then use extract_completion_versions for filtering and sorting
    let stripped_versions: Vec<String> = versions
        .iter()
        .map(|v| v.trim_start_matches('v').to_string())
        .collect();

    let items = version_trimmed
        .extract_completion_versions(stripped_versions.into_iter())
        .into_iter()
        .take(MAXIMUM_VERSIONS_SHOWN)
        .enumerate()
        .map(|(index, pv)| {
            // Add v prefix back for go.mod format
            let display = format!("v{}", pv.item_version_raw);
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

fn complete_name(path: &str, range: Range) -> ServerResult<Option<CompletionResponse>> {
    let packages = top_go_packages_prefixed(path, MAXIMUM_PACKAGES_SHOWN)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let items = packages
        .into_iter()
        .map(|package| CompletionItem {
            label: package.path.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            detail: Some(package.description.to_string()),
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

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
use tracing::debug;

use parser::npm;
use shared::Versioned;

use super::Clients;
use super::constants::top_npm_packages_prefixed;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;

pub async fn get_npm_completions(
    clients: &Clients,
    doc: &Document,
    pos: Position,
    node: Node<'_>,
) -> ServerResult<Option<CompletionResponse>> {
    let Some(dep) = npm::parse_dependency(node) else {
        return Ok(None);
    };

    let (name, spec) = dep.text(doc);
    if spec.starts_with("file:") || spec.starts_with("github:") || spec.starts_with("git+") {
        return Ok(None); // Ignore these spec formats, for now
    }

    // Try to complete specs (versions)
    if ts_range_contains_lsp_position(dep.spec.range(), pos) {
        debug!("Completing version: {dep:?}");
        return complete_spec(
            clients,
            name.as_str(),
            spec.as_str(),
            ts_range_to_lsp_range(dep.spec.range()),
        )
        .await;
    }

    // Try to complete names
    if ts_range_contains_lsp_position(dep.name.range(), pos) {
        debug!("Completing name: {dep:?}");
        return complete_name(name.as_str(), ts_range_to_lsp_range(dep.name.range()));
    }

    // No completions yet - probably empty spec
    Ok(None)
}

fn complete_name(name: &str, range: Range) -> ServerResult<Option<CompletionResponse>> {
    let packages = top_npm_packages_prefixed(name, MAXIMUM_PACKAGES_SHOWN)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let items = packages
        .into_iter()
        .map(|package| CompletionItem {
            label: package.name.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: package.name.to_string(),
                range: range.shrink(1, 1),
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

async fn complete_spec(
    clients: &Clients,
    name: &str,
    spec: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let metadata = match clients.npm.get_registry_metadata(name).await {
        Err(_) => return Ok(None),
        Ok(m) => m,
    };

    let valid_vec = spec
        .extract_completion_versions(metadata.versions.into_values())
        .into_iter()
        .take(MAXIMUM_PACKAGES_SHOWN)
        .enumerate()
        .map(|(index, potential_version)| CompletionItem {
            label: potential_version.item_version_raw.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{index:0>5}")),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: potential_version.item_version_raw.to_string(),
                range: range.shrink(1, 1),
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(valid_vec)))
}

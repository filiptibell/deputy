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

use crate::parser::rokit;
use crate::util::Versioned;

use super::constants::{top_rokit_tool_authors_prefixed, top_rokit_tool_names_prefixed};
use super::Clients;

const MAXIMUM_TOOLS_SHOWN: usize = 64;

pub async fn get_rokit_completions(
    clients: &Clients,
    doc: &Document,
    pos: Position,
    node: Node<'_>,
) -> ServerResult<Option<CompletionResponse>> {
    let Some(dep) = rokit::parse_dependency(node) else {
        return Ok(None);
    };

    let ranges = dep.spec_ranges(doc);
    let (owner, repository, version) = ranges.text(doc);

    // Try to complete versions
    if let Some(range) = ranges.version {
        if ts_range_contains_lsp_position(range, pos) {
            debug!("Completing version: {dep:?}");
            return complete_version(
                clients,
                owner.unwrap_or_default(),
                repository.unwrap_or_default(),
                version.unwrap_or_default(),
                ts_range_to_lsp_range(range),
            )
            .await;
        }
    }

    // Try to complete names
    if let Some(range) = ranges.repository {
        if ts_range_contains_lsp_position(range, pos) {
            debug!("Completing name: {dep:?}");
            return complete_repository(
                owner.unwrap_or_default(),
                repository.unwrap_or_default(),
                ts_range_to_lsp_range(range),
            );
        }
    }

    // Try to complete authors
    if let Some(range) = ranges.owner {
        if ts_range_contains_lsp_position(range, pos) {
            debug!("Completing author: {dep:?}");
            return complete_owner(owner.unwrap_or_default(), ts_range_to_lsp_range(range));
        }
    }

    // No completions yet - probably empty spec
    Ok(None)
}

fn complete_owner(author: &str, range: Range) -> ServerResult<Option<CompletionResponse>> {
    let items = top_rokit_tool_authors_prefixed(author, MAXIMUM_TOOLS_SHOWN)
        .into_iter()
        .map(|item| CompletionItem {
            label: item.name.to_string(),
            kind: Some(CompletionItemKind::ENUM),
            commit_characters: Some(vec![String::from("/")]),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: item.name.to_string(),
                range,
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

fn complete_repository(
    author: &str,
    name: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let items = top_rokit_tool_names_prefixed(author, name, MAXIMUM_TOOLS_SHOWN)
        .into_iter()
        .map(|item| CompletionItem {
            label: item.name.to_string(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            commit_characters: Some(vec![String::from("@")]),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: item.name.to_string(),
                range,
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

async fn complete_version(
    clients: &Clients,
    author: &str,
    name: &str,
    version: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let metadatas = match clients.github.get_repository_releases(author, name).await {
        Err(_) => return Ok(None),
        Ok(m) => m,
    };

    let valid_vec = version
        .extract_completion_versions(metadatas.into_iter())
        .into_iter()
        .take(MAXIMUM_TOOLS_SHOWN)
        .enumerate()
        .map(|(index, potential_version)| CompletionItem {
            label: potential_version.item_version_raw.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{index:0>5}")),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: potential_version.item_version_raw.to_string(),
                range,
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(valid_vec)))
}

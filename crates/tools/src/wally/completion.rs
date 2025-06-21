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

use parser::wally;
use versioning::Versioned;

use crate::shared::filter_starts_with;

use super::Clients;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;

pub async fn get_wally_completions(
    clients: &Clients,
    doc: &Document,
    pos: Position,
    index_url: &str,
    node: Node<'_>,
) -> ServerResult<Option<CompletionResponse>> {
    let Some(dep) = wally::parse_dependency(node) else {
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
                index_url,
                owner.unwrap_or_default(),
                repository.unwrap_or_default(),
                version.unwrap_or_default(),
                ts_range_to_lsp_range(range),
            )
            .await;
        }
    }

    // Try to complete packages
    if let Some(range) = ranges.repository {
        if ts_range_contains_lsp_position(range, pos) {
            debug!("Completing name: {dep:?}");
            return complete_package(
                clients,
                index_url,
                owner.unwrap_or_default(),
                repository.unwrap_or_default(),
                ts_range_to_lsp_range(range),
            )
            .await;
        }
    }

    // Try to complete scopes
    if let Some(range) = ranges.owner {
        if ts_range_contains_lsp_position(range, pos) {
            debug!("Completing scope: {dep:?}");
            return complete_scope(
                clients,
                index_url,
                owner.unwrap_or_default(),
                ts_range_to_lsp_range(range),
            )
            .await;
        }
    }

    // No completions yet - probably empty spec
    Ok(None)
}

async fn complete_scope(
    clients: &Clients,
    index_url: &str,
    scope: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let package_scopes = match clients.wally.get_index_scopes(index_url).await {
        Err(_) => return Ok(None),
        Ok(m) => m,
    };

    let items = package_scopes
        .into_iter()
        .filter(|s| filter_starts_with(s.as_str(), scope))
        .take(MAXIMUM_PACKAGES_SHOWN)
        .map(|scope| CompletionItem {
            label: scope.to_string(),
            kind: Some(CompletionItemKind::ENUM),
            commit_characters: Some(vec![String::from("/")]),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: scope.to_string(),
                range,
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

async fn complete_package(
    clients: &Clients,
    index_url: &str,
    author: &str,
    package: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let package_names = match clients.wally.get_index_packages(index_url, author).await {
        Err(_) => return Ok(None),
        Ok(m) => m,
    };

    let items = package_names
        .into_iter()
        .filter(|p| filter_starts_with(p.as_str(), package))
        .take(MAXIMUM_PACKAGES_SHOWN)
        .map(|package| CompletionItem {
            label: package.to_string(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            commit_characters: Some(vec![String::from("@")]),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: package.to_string(),
                range,
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

async fn complete_version(
    clients: &Clients,
    index_url: &str,
    author: &str,
    package: &str,
    version: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let metadatas = match clients
        .wally
        .get_index_metadatas(index_url, author, package)
        .await
    {
        Err(_) => return Ok(None),
        Ok(m) => m,
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
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: potential_version.item_version_raw.to_string(),
                range,
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(valid_vec)))
}

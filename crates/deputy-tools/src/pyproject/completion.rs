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

use deputy_parser::pyproject;
use deputy_versioning::PepVersioned;

use super::Clients;
use super::constants::top_pypi_packages_prefixed;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;

pub async fn get_pyproject_completions(
    clients: &Clients,
    doc: &Document,
    pos: Position,
    node: Node<'_>,
) -> ServerResult<Option<CompletionResponse>> {
    let Some(dep) = pyproject::parse_dependency(node) else {
        return Ok(None);
    };

    let ranges = dep.spec_ranges(doc);
    let (name, version) = dep.text(doc);

    // Try to complete versions
    if let Some(range) = ranges.version
        && ts_range_contains_lsp_position(range, pos)
    {
        debug!("Completing version: {dep:?}");
        return complete_version(
            clients,
            name.unwrap_or_default(),
            version.unwrap_or_default(),
            ts_range_to_lsp_range(range),
        )
        .await;
    }

    // Try to complete extras
    for extra_range in dep.extra_ranges(doc) {
        if ts_range_contains_lsp_position(extra_range, pos) {
            let txt = doc.text();
            let extra_text = txt
                .byte_slice(extra_range.start_byte..extra_range.end_byte)
                .as_str()
                .unwrap_or_default();
            debug!("Completing extras: {dep:?}");
            return complete_extras(
                clients,
                name.as_deref().unwrap_or_default(),
                extra_text,
                ts_range_to_lsp_range(extra_range),
            )
            .await;
        }
    }

    // Try to complete names
    if let Some(range) = ranges.name
        && ts_range_contains_lsp_position(range, pos)
    {
        debug!("Completing name: {dep:?}");
        return complete_name(name.unwrap_or_default(), ts_range_to_lsp_range(range));
    }

    // No completions yet - probably empty spec
    Ok(None)
}

fn complete_name(name: impl AsRef<str>, range: Range) -> ServerResult<Option<CompletionResponse>> {
    let packages = top_pypi_packages_prefixed(name.as_ref(), MAXIMUM_PACKAGES_SHOWN)
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
                range,
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

async fn complete_extras(
    clients: &Clients,
    name: &str,
    current: &str,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let Ok(meta) = clients.pypi.get_registry_metadata(name).await else {
        return Ok(None);
    };
    let Some(extras) = meta.info.provides_extra else {
        return Ok(None);
    };

    let items = extras
        .into_iter()
        .filter(|e| e.starts_with(current))
        .enumerate()
        .map(|(index, extra)| CompletionItem {
            label: extra.clone(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{index:0>5}")),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: extra,
                range,
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(items)))
}

async fn complete_version(
    clients: &Clients,
    name: impl AsRef<str>,
    version: impl AsRef<str>,
    range: Range,
) -> ServerResult<Option<CompletionResponse>> {
    let Ok(metadata) = clients.pypi.get_simple_metadata(name.as_ref()).await else {
        return Ok(None);
    };

    let versions = metadata.versions();
    let valid_vec = version
        .as_ref()
        .extract_completion_versions(versions.into_iter())
        .into_iter()
        .take(MAXIMUM_PACKAGES_SHOWN)
        .enumerate()
        .map(|(index, potential_version)| CompletionItem {
            label: potential_version.item_version_raw.clone(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{index:0>5}")),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                new_text: potential_version.item_version_raw.clone(),
                range,
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(valid_vec)))
}

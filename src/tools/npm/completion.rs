use async_language_server::{
    lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse},
    server::{Document, ServerResult},
};

use crate::parser::Dependency;
use crate::util::Versioned;

use super::constants::top_npm_packages_prefixed;
use super::Clients;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;

pub async fn get_npm_completions_name(
    _clients: &Clients,
    _document: &Document,
    dep: &Dependency,
) -> ServerResult<Option<CompletionResponse>> {
    let dname = dep.name().unquoted();

    let packages = top_npm_packages_prefixed(dname, MAXIMUM_PACKAGES_SHOWN)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let items = packages
        .into_iter()
        .map(|package| CompletionItem {
            label: package.name.to_string(),
            kind: Some(CompletionItemKind::VALUE),

            detail: None,
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

pub async fn get_npm_completions_version(
    clients: &Clients,
    _document: &Document,
    dep: &Dependency,
) -> ServerResult<Option<CompletionResponse>> {
    let name = dep.name().unquoted();

    let metadata = match clients.npm.get_registry_metadata(name).await {
        Err(_) => return Ok(None),
        Ok(m) => m,
    };

    let valid_vec = dep
        .extract_completion_versions(metadata.versions.into_values())
        .into_iter()
        .take(MAXIMUM_PACKAGES_SHOWN)
        .enumerate()
        .map(|(index, potential_version)| CompletionItem {
            label: potential_version.item_version_raw.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{:0>5}", index)),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(Some(CompletionResponse::Array(valid_vec)))
}

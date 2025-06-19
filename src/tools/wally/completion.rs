use async_language_server::{
    lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse},
    server::{Document, ServerResult},
};

use crate::parser::SimpleDependency;
use crate::tools::shared::filter_starts_with;
use crate::util::Versioned;

use super::Clients;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;

pub async fn get_wally_completions_spec_author(
    clients: &Clients,
    _document: &Document,
    index_url: &str,
    dep: &SimpleDependency,
) -> ServerResult<Option<CompletionResponse>> {
    let dep = dep.parsed_spec();
    let author = &dep.author;

    let package_scopes = match clients.wally.get_index_scopes(index_url).await {
        Err(_) => return Ok(None),
        Ok(m) => m,
    };

    let items = package_scopes
        .into_iter()
        .filter(|package| filter_starts_with(package.as_str(), author.unquoted()))
        .take(MAXIMUM_PACKAGES_SHOWN)
        .map(|package| CompletionItem {
            label: package.to_string(),
            kind: Some(CompletionItemKind::ENUM),
            commit_characters: Some(vec![String::from("/")]),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

pub async fn get_wally_completions_spec_name(
    clients: &Clients,
    _document: &Document,
    index_url: &str,
    dep: &SimpleDependency,
) -> ServerResult<Option<CompletionResponse>> {
    let dep = dep.parsed_spec();
    let author = &dep.author;

    let Some(name) = dep.name.as_ref() else {
        return Ok(None);
    };

    let package_names = match clients
        .wally
        .get_index_packages(index_url, author.unquoted())
        .await
    {
        Err(_) => return Ok(None),
        Ok(m) => m,
    };

    let items = package_names
        .into_iter()
        .filter(|package| filter_starts_with(package.as_str(), name.unquoted()))
        .take(MAXIMUM_PACKAGES_SHOWN)
        .map(|package| CompletionItem {
            label: package.to_string(),
            kind: Some(CompletionItemKind::ENUM_MEMBER),
            commit_characters: Some(vec![String::from("@")]),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

pub async fn get_wally_completions_spec_version(
    clients: &Clients,
    _document: &Document,
    index_url: &str,
    dep: &SimpleDependency,
) -> ServerResult<Option<CompletionResponse>> {
    let dep = dep.parsed_spec();
    let author = &dep.author;

    let Some(name) = dep.name.as_ref() else {
        return Ok(None);
    };
    let Some(version) = dep.version.as_ref() else {
        return Ok(None);
    };

    let metadatas = match clients
        .wally
        .get_index_metadatas(index_url, author.unquoted(), name.unquoted())
        .await
    {
        Err(_) => return Ok(None),
        Ok(m) => m,
    };

    let valid_vec = version
        .unquoted()
        .extract_completion_versions(metadatas.into_iter())
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

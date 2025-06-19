use async_language_server::{
    lsp_types::{CompletionItem, CompletionItemKind, CompletionResponse},
    server::{Document, ServerResult},
};

use crate::parser::SimpleDependency;
use crate::util::Versioned;

use super::constants::{top_rokit_tool_authors_prefixed, top_rokit_tool_names_prefixed};
use super::Clients;

const MAXIMUM_TOOLS_SHOWN: usize = 64;

pub async fn get_rokit_completions_spec_author(
    _clients: &Clients,
    _document: &Document,
    dep: &SimpleDependency,
) -> ServerResult<Option<CompletionResponse>> {
    let dep = dep.parsed_spec();
    let author = &dep.author;

    let items = top_rokit_tool_authors_prefixed(author.unquoted(), MAXIMUM_TOOLS_SHOWN)
        .into_iter()
        .map(|item| CompletionItem {
            label: item.name.to_string(),
            kind: Some(CompletionItemKind::ENUM),
            commit_characters: Some(vec![String::from("/")]),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

pub async fn get_rokit_completions_spec_name(
    _clients: &Clients,
    _document: &Document,
    dep: &SimpleDependency,
) -> ServerResult<Option<CompletionResponse>> {
    let dep = dep.parsed_spec();
    let author = &dep.author;

    let Some(name) = dep.name.as_ref() else {
        return Ok(None);
    };

    let items =
        top_rokit_tool_names_prefixed(author.unquoted(), name.unquoted(), MAXIMUM_TOOLS_SHOWN)
            .into_iter()
            .map(|item| CompletionItem {
                label: item.name.to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                commit_characters: Some(vec![String::from("@")]),
                ..Default::default()
            })
            .collect::<Vec<_>>();
    Ok(Some(CompletionResponse::Array(items)))
}

pub async fn get_rokit_completions_spec_version(
    clients: &Clients,
    _document: &Document,
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
        .github
        .get_repository_releases(author.unquoted(), name.unquoted())
        .await
    {
        Err(_) => return Ok(None),
        Ok(m) => m,
    };

    let valid_vec = version
        .unquoted()
        .extract_completion_versions(metadatas.into_iter())
        .into_iter()
        .take(MAXIMUM_TOOLS_SHOWN)
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

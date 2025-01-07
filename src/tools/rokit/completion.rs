use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::clients::*;
use crate::parser::SimpleDependency;
use crate::server::*;

use super::constants::{top_rokit_tool_authors_prefixed, top_rokit_tool_names_prefixed};
use super::Versioned;

const MAXIMUM_TOOLS_SHOWN: usize = 64;

pub async fn get_rokit_completions_spec_author(
    _clients: &Clients,
    document: &Document,
    dep: &SimpleDependency,
) -> Result<CompletionResponse> {
    let dep = dep.parsed_spec();
    let author = &dep.author;

    let items = top_rokit_tool_authors_prefixed(author.unquoted(), MAXIMUM_TOOLS_SHOWN)
        .into_iter()
        .map(|item| CompletionItem {
            label: item.name.to_string(),
            kind: Some(CompletionItemKind::ENUM),
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(author.range, item.name.to_string()),
            )),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(CompletionResponse::Array(items))
}

pub async fn get_rokit_completions_spec_name(
    _clients: &Clients,
    document: &Document,
    dep: &SimpleDependency,
) -> Result<CompletionResponse> {
    let dep = dep.parsed_spec();
    let author = &dep.author;

    let Some(name) = dep.name.as_ref() else {
        return Ok(CompletionResponse::Array(Vec::new()));
    };

    let items =
        top_rokit_tool_names_prefixed(author.unquoted(), name.unquoted(), MAXIMUM_TOOLS_SHOWN)
            .into_iter()
            .map(|item| CompletionItem {
                label: item.name.to_string(),
                kind: Some(CompletionItemKind::ENUM_MEMBER),
                text_edit: Some(CompletionTextEdit::Edit(
                    document.create_edit(name.range, item.name.to_string()),
                )),
                ..Default::default()
            })
            .collect::<Vec<_>>();
    Ok(CompletionResponse::Array(items))
}

pub async fn get_rokit_completions_spec_version(
    clients: &Clients,
    document: &Document,
    dep: &SimpleDependency,
) -> Result<CompletionResponse> {
    let dep = dep.parsed_spec();
    let author = &dep.author;

    let Some(name) = dep.name.as_ref() else {
        return Ok(CompletionResponse::Array(Vec::new()));
    };
    let Some(version) = dep.version.as_ref() else {
        return Ok(CompletionResponse::Array(Vec::new()));
    };

    let metadatas = match clients
        .github
        .get_repository_releases(author.unquoted(), name.unquoted())
        .await
    {
        Err(_) => return Ok(CompletionResponse::Array(Vec::new())),
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
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(version.range, potential_version.item_version_raw),
            )),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(CompletionResponse::Array(valid_vec))
}

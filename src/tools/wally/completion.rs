use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::clients::*;
use crate::parser::SimpleDependency;
use crate::server::*;

use super::Versioned;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;

pub async fn get_wally_completions_spec_author(
    clients: &Clients,
    document: &Document,
    index_url: &str,
    dep: &SimpleDependency,
) -> Result<CompletionResponse> {
    let dep = dep.parsed_spec();
    let author = &dep.author;

    let package_scopes = match clients.wally.get_index_scopes(index_url).await {
        Err(_) => return Ok(CompletionResponse::Array(Vec::new())),
        Ok(m) => m,
    };

    let items = package_scopes
        .into_iter()
        .filter(|package| package.trim().starts_with(author.unquoted()))
        .take(MAXIMUM_PACKAGES_SHOWN)
        .map(|package| CompletionItem {
            label: package.to_string(),
            kind: Some(CompletionItemKind::ENUM),
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(author.range, package.to_string()),
            )),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(CompletionResponse::Array(items))
}

pub async fn get_wally_completions_spec_name(
    clients: &Clients,
    document: &Document,
    index_url: &str,
    dep: &SimpleDependency,
) -> Result<CompletionResponse> {
    let dep = dep.parsed_spec();
    let author = &dep.author;

    let Some(name) = dep.name.as_ref() else {
        return Ok(CompletionResponse::Array(Vec::new()));
    };

    let package_names = match clients
        .wally
        .get_index_packages(index_url, author.unquoted())
        .await
    {
        Err(_) => return Ok(CompletionResponse::Array(Vec::new())),
        Ok(m) => m,
    };

    let items = package_names
        .into_iter()
        .filter(|package| package.trim().starts_with(name.unquoted()))
        .take(MAXIMUM_PACKAGES_SHOWN)
        .map(|package| CompletionItem {
            label: package.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(name.range, package.to_string()),
            )),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(CompletionResponse::Array(items))
}

pub async fn get_wally_completions_spec_version(
    clients: &Clients,
    document: &Document,
    index_url: &str,
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
        .wally
        .get_index_metadatas(index_url, author.unquoted(), name.unquoted())
        .await
    {
        Err(_) => return Ok(CompletionResponse::Array(Vec::new())),
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
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(version.range, potential_version.item_version_raw),
            )),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(CompletionResponse::Array(valid_vec))
}

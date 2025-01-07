use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::clients::*;
use crate::parser::Dependency;
use crate::server::*;

use super::constants::top_npm_packages_prefixed;
use super::Versioned;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;

pub async fn get_npm_completions_name(
    _clients: &Clients,
    document: &Document,
    dep: &Dependency,
) -> Result<CompletionResponse> {
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
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(dep.name().range, package.name.to_string()),
            )),
            detail: None,
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(CompletionResponse::Array(items))
}

pub async fn get_npm_completions_version(
    clients: &Clients,
    document: &Document,
    dep: &Dependency,
) -> Result<CompletionResponse> {
    let name = dep.name().unquoted();
    let Some(version) = dep.spec().and_then(|s| s.contents.version.as_ref()) else {
        return Ok(CompletionResponse::Array(Vec::new()));
    };

    let metadata = match clients.npm.get_registry_metadata(name).await {
        Err(_) => return Ok(CompletionResponse::Array(Vec::new())),
        Ok(m) => m,
    };

    let valid_vec = dep
        .extract_completion_versions(metadata.versions.into_values())
        .into_iter()
        .enumerate()
        .map(|(index, potential_version)| CompletionItem {
            label: potential_version.item_version_raw.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{:0>5}", index)),
            text_edit: Some(CompletionTextEdit::Edit(document.create_substring_edit(
                version.range.start.line,
                potential_version.this_version_raw,
                potential_version.item_version_raw,
            ))),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(CompletionResponse::Array(valid_vec))
}

use semver::Version;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::clients::*;
use crate::parser::Dependency;
use crate::server::*;

use super::constants::top_npm_packages_prefixed;
use super::strip_specifiers;

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

    let valid_metadatas_with_versions = metadata
        .versions
        .into_values()
        .filter_map(|metadata| match Version::parse(&metadata.version) {
            Ok(version) => Some((metadata, version)),
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    let mut valid_metadatas = valid_metadatas_with_versions
        .into_iter()
        .filter_map(|(metadata, metadata_version)| {
            let dep_version = strip_specifiers(version.unquoted());
            let met_version = metadata_version.to_string();
            tracing::debug!(
                "CMP {dep_version} WITH {met_version}: {} {}",
                dep_version.is_empty(),
                (dep_version.len() <= met_version.len() && met_version.starts_with(dep_version))
            );
            if dep_version.is_empty()
                || (dep_version.len() <= met_version.len() && met_version.starts_with(dep_version))
            {
                Some((metadata, metadata_version))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    valid_metadatas.sort_by(|left, right| right.1.cmp(&left.1));

    let valid_vec = valid_metadatas
        .into_iter()
        .enumerate()
        .map(|(index, (_, meta_version))| CompletionItem {
            label: meta_version.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{:0>5}", index)),
            text_edit: Some(CompletionTextEdit::Edit(document.create_substring_edit(
                version.range.start.line,
                strip_specifiers(version.unquoted()),
                meta_version.to_string(),
            ))),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(CompletionResponse::Array(valid_vec))
}

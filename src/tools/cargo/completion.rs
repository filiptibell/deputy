use semver::Version;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tracing::debug;

use crate::clients::*;
use crate::parser::Dependency;
use crate::server::*;
use crate::tools::cargo::constants::CratesIoPackage;

use super::constants::top_crates_io_packages_prefixed;

const MAXIMUM_PACKAGES_SHOWN: usize = 64;
const MINIMUM_PACKAGES_BEFORE_FETCH: usize = 16; // Less than 16 packages found statically = fetch dynamically

pub async fn get_cargo_completions_name(
    clients: &Clients,
    document: &Document,
    dep: &Dependency,
) -> Result<CompletionResponse> {
    let dname = dep.name().unquoted();

    let mut packages = top_crates_io_packages_prefixed(dname, MAXIMUM_PACKAGES_SHOWN)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    if packages.len() < MINIMUM_PACKAGES_BEFORE_FETCH {
        if let Ok(crates) = clients.crates.search_crates(dname).await {
            let count_prev = packages.len();

            packages.extend(crates.inner.into_iter().map(|m| CratesIoPackage {
                name: m.name.to_string().into(),
                downloads: m.downloads.total_count,
                description: m.description.to_string().into(),
            }));

            packages.sort_by_key(|package| package.name.to_ascii_lowercase());
            packages.dedup_by_key(|p| p.name.to_ascii_lowercase());
            packages.truncate(MINIMUM_PACKAGES_BEFORE_FETCH);

            let count_after = packages.len();
            if count_after > count_prev {
                debug!(
                    "Found {} additional crates for prefix '{dname}'",
                    count_after.saturating_sub(count_prev),
                );
            }
        }
    }

    let items = packages
        .into_iter()
        .map(|package| CompletionItem {
            label: package.name.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(dep.name().range, package.name.to_string()),
            )),
            detail: Some(package.description.to_string()),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(CompletionResponse::Array(items))
}

pub async fn get_cargo_completions_version(
    clients: &Clients,
    document: &Document,
    dep: &Dependency,
) -> Result<CompletionResponse> {
    let name = dep.name().unquoted();
    let Some(version) = dep.spec().and_then(|s| s.contents.version.as_ref()) else {
        return Ok(CompletionResponse::Array(Vec::new()));
    };

    let metadatas = match clients.crates.get_sparse_index_crate_metadatas(name).await {
        Err(_) => return Ok(CompletionResponse::Array(Vec::new())),
        Ok(m) => m,
    };

    let valid_metadatas_with_versions = metadatas
        .into_iter()
        .filter_map(|metadata| match Version::parse(&metadata.version) {
            Ok(version) => Some((metadata, version)),
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    let mut valid_metadatas = valid_metadatas_with_versions
        .into_iter()
        .filter_map(|(metadata, metadata_version)| {
            let dep_version = version.unquoted();
            let met_version = metadata_version.to_string();
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
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(version.range, meta_version.to_string()),
            )),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    Ok(CompletionResponse::Array(valid_vec))
}

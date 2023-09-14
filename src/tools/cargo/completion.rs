use std::ops::Range as StdRange;

use semver::Version;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::clients::*;
use crate::server::*;

async fn complete_package_version(
    clients: &Clients,
    document: &Document,
    range: StdRange<usize>,
    name: &str,
    version: &str,
) -> Result<Vec<CompletionItem>> {
    let metadatas = match clients.crates.get_index_metadatas(name).await {
        Err(_) => return Ok(Vec::new()),
        Ok(m) => m,
    };

    let mut valid_metadatas = metadatas
        .into_iter()
        .filter_map(|metadata| {
            let ver = metadata.version.to_ascii_lowercase();
            let smallest_len = version.len().min(ver.len());
            if version.is_empty()
                || version[..smallest_len].eq_ignore_ascii_case(&ver[..smallest_len])
            {
                if let Ok(version) = Version::parse(&ver) {
                    Some((version, metadata))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    valid_metadatas.sort_by(|left, right| right.0.cmp(&left.0));

    Ok(valid_metadatas
        .into_iter()
        .enumerate()
        .map(|(index, (version, _))| CompletionItem {
            label: version.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{:0>5}", index)),
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(range.clone(), version.to_string()),
            )),
            ..Default::default()
        })
        .collect())
}

pub async fn get_package_completions(
    clients: &Clients,
    document: &Document,
    range: StdRange<usize>,
    name: &str,
    version: &str,
) -> Result<CompletionResponse> {
    let items = complete_package_version(clients, document, range, name, version).await?;
    Ok(CompletionResponse::Array(items))
}

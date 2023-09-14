use std::ops::Range as StdRange;

use semver::Version;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::clients::*;
use crate::server::*;

async fn complete_package_author(
    clients: &Clients,
    document: &Document,
    index_url: &str,
    replace_range: StdRange<usize>,
    author: &str,
) -> Result<Vec<CompletionItem>> {
    let authors = match clients.wally.get_index_scopes(index_url).await {
        Err(_) => return Ok(Vec::new()),
        Ok(a) => a,
    };

    let mut valid_authors = authors
        .into_iter()
        .filter_map(|a| {
            let smallest_len = author.len().min(a.len());
            if author.is_empty() || author[..smallest_len].eq_ignore_ascii_case(&a[..smallest_len])
            {
                Some(a)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    valid_authors.sort();

    Ok(valid_authors
        .into_iter()
        .enumerate()
        .map(|(index, author)| CompletionItem {
            label: author.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{:0>5}", index)),
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(replace_range.clone(), author.to_string()),
            )),
            ..Default::default()
        })
        .collect())
}

async fn complete_package_name(
    clients: &Clients,
    document: &Document,
    index_url: &str,
    replace_range: StdRange<usize>,
    author: &str,
    name: &str,
) -> Result<Vec<CompletionItem>> {
    let names = match clients.wally.get_index_packages(index_url, author).await {
        Err(_) => return Ok(Vec::new()),
        Ok(n) => n,
    };

    let mut valid_names = names
        .into_iter()
        .filter_map(|n| {
            let smallest_len = name.len().min(n.len());
            if name.is_empty() || name[..smallest_len].eq_ignore_ascii_case(&n[..smallest_len]) {
                Some(n)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    valid_names.sort();

    Ok(valid_names
        .into_iter()
        .enumerate()
        .map(|(index, name)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{:0>5}", index)),
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(replace_range.clone(), name.to_string()),
            )),
            ..Default::default()
        })
        .collect())
}

async fn complete_package_version(
    clients: &Clients,
    document: &Document,
    index_url: &str,
    replace_range: StdRange<usize>,
    author: &str,
    name: &str,
    version: &str,
) -> Result<Vec<CompletionItem>> {
    let metadatas = match clients
        .wally
        .get_index_metadatas(index_url, author, name)
        .await
    {
        Err(_) => return Ok(Vec::new()),
        Ok(m) => m,
    };

    let mut valid_metadatas = metadatas
        .into_iter()
        .filter_map(|metadata| {
            let ver = metadata.package.version.to_ascii_lowercase();
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
                document.create_edit(replace_range.clone(), version.to_string()),
            )),
            ..Default::default()
        })
        .collect())
}

pub async fn get_package_completions(
    clients: &Clients,
    document: &Document,
    index_url: &str,
    replace_range: StdRange<usize>,
    substring: &str,
) -> Result<CompletionResponse> {
    let idx_slash = substring.find('/');
    let idx_at = idx_slash
        .and_then(|idx| substring[idx..].find('@'))
        .map(|idx| idx + idx_slash.unwrap());
    let items = match (idx_slash, idx_at) {
        (Some(idx_slash), Some(idx_at)) => {
            let author = &substring[..idx_slash];
            let name = &substring[idx_slash + 1..idx_at];
            let version = &substring[idx_at + 1..];
            let range = StdRange {
                start: replace_range.start + idx_at + 1,
                end: replace_range.end,
            };
            complete_package_version(clients, document, index_url, range, author, name, version)
                .await
        }
        (Some(idx_slash), _) => {
            let author = &substring[..idx_slash];
            let name = &substring[idx_slash + 1..];
            let range = StdRange {
                start: replace_range.start + idx_slash + 1,
                end: replace_range.end,
            };
            complete_package_name(clients, document, index_url, range, author, name).await
        }
        _ => complete_package_author(clients, document, index_url, replace_range, substring).await,
    }?;
    Ok(CompletionResponse::Array(items))
}

use std::ops::Range as StdRange;

use semver::Version;

use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::*;

use crate::github::*;
use crate::server::*;
use crate::util::*;

use super::constants::*;

async fn complete_tool_author(
    _github: &GithubWrapper,
    document: &Document,
    replace_range: StdRange<usize>,
    author: &str,
) -> Result<Vec<CompletionItem>> {
    Ok(KNOWN_TOOLS
        .keys()
        .filter(|a| author.is_empty() || author.eq_ignore_ascii_case(a))
        .map(|a| CompletionItem {
            label: a.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(replace_range.clone(), a.to_string()),
            )),
            ..Default::default()
        })
        .collect())
}

async fn complete_tool_name(
    _github: &GithubWrapper,
    document: &Document,
    replace_range: StdRange<usize>,
    author: &str,
    name: &str,
) -> Result<Vec<CompletionItem>> {
    let key = KNOWN_TOOLS.keys().find(|k| author.eq_ignore_ascii_case(k));
    match key.and_then(|k| KNOWN_TOOLS.get(k)) {
        None => Ok(Vec::new()),
        Some(v) => Ok(v
            .iter()
            .filter(|repo| name.is_empty() || name.eq_ignore_ascii_case(repo))
            .map(|repo| CompletionItem {
                label: repo.to_string(),
                kind: Some(CompletionItemKind::VALUE),
                text_edit: Some(CompletionTextEdit::Edit(
                    document.create_edit(replace_range.clone(), repo.to_string()),
                )),
                ..Default::default()
            })
            .collect()),
    }
}

async fn complete_tool_version(
    github: &GithubWrapper,
    document: &Document,
    replace_range: StdRange<usize>,
    author: &str,
    name: &str,
    version: &str,
) -> Result<Vec<CompletionItem>> {
    let releases = match github.get_repository_releases(author, name).await {
        Err(e) if e.is_rate_limit_error() || e.is_not_found_error() => return Ok(Vec::new()),
        Err(_) => return Err(Error::invalid_request()),
        Ok(v) => v,
    };

    let mut valid_releases = releases
        .into_iter()
        .filter_map(|release| {
            let tag = release
                .tag_name
                .trim_start_matches('v')
                .to_ascii_lowercase();
            let smallest_len = version.len().min(tag.len());
            if version.is_empty()
                || version[..smallest_len].eq_ignore_ascii_case(&tag[..smallest_len])
            {
                if let Ok(version) = Version::parse(&tag) {
                    Some((version, release))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    valid_releases.sort_by(|left, right| left.0.cmp(&right.0));
    valid_releases.reverse();

    Ok(valid_releases
        .into_iter()
        .enumerate()
        .map(|(index, (version, release))| CompletionItem {
            label: version.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            sort_text: Some(format!("{:0>5}", index)),
            text_edit: Some(CompletionTextEdit::Edit(
                document.create_edit(replace_range.clone(), version.to_string()),
            )),
            label_details: Some(CompletionItemLabelDetails {
                description: release.published_at,
                detail: None,
            }),
            ..Default::default()
        })
        .collect())
}

pub async fn get_tool_completions(
    github: &GithubWrapper,
    document: &Document,
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
            complete_tool_version(github, document, range, author, name, version).await
        }
        (Some(idx_slash), _) => {
            let author = &substring[..idx_slash];
            let name = &substring[idx_slash + 1..];
            let range = StdRange {
                start: replace_range.start + idx_slash + 1,
                end: replace_range.end,
            };
            complete_tool_name(github, document, range, author, name).await
        }
        _ => complete_tool_author(github, document, replace_range, substring).await,
    }?;
    Ok(CompletionResponse::Array(items))
}

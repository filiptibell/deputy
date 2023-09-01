use std::ops::Range as StdRange;

use semver::Version;

use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::*;

use crate::github::*;
use crate::server::*;

use super::constants::*;

async fn complete_tool_author(
    _github: &GithubWrapper,
    document: &Document,
    replace_range: StdRange<usize>,
    author: &str,
) -> Result<Vec<CompletionItem>> {
    let author_low = author.to_ascii_lowercase();
    Ok(KNOWN_TOOLS
        .keys()
        .filter(|a| author_low.is_empty() || a.to_ascii_lowercase().starts_with(&author_low))
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
    let author_low = author.to_ascii_lowercase();
    let name_low = name.to_ascii_lowercase();
    let key = KNOWN_TOOLS
        .keys()
        .find(|k| k.to_ascii_lowercase() == author_low);
    match key.and_then(|k| KNOWN_TOOLS.get(k)) {
        None => Ok(Vec::new()),
        Some(v) => Ok(v
            .iter()
            .filter(|repo| name_low.is_empty() || repo.to_ascii_lowercase().starts_with(&name_low))
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
    let version_low = version.to_ascii_lowercase();
    match github.get_repository_releases(author, name).await {
        Err(e) if e.is_rate_limit_error() || e.is_not_found_error() => Ok(Vec::new()),
        Err(_) => Err(Error::invalid_request()),
        Ok(v) => {
            let mut tags = v
                .into_iter()
                .map(|release| {
                    release
                        .tag_name
                        .trim_start_matches('v')
                        .to_ascii_lowercase()
                })
                .filter_map(|tag| {
                    if version_low.is_empty() || tag.starts_with(&version_low) {
                        Version::parse(&tag).ok()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            tags.sort();
            tags.reverse();
            Ok(tags
                .into_iter()
                .enumerate()
                .map(|(index, tag)| CompletionItem {
                    label: tag.to_string(),
                    kind: Some(CompletionItemKind::VALUE),
                    sort_text: Some(format!("{:0>5}", index)),
                    text_edit: Some(CompletionTextEdit::Edit(
                        document.create_edit(replace_range.clone(), tag.to_string()),
                    )),
                    ..Default::default()
                })
                .collect())
        }
    }
}

pub async fn get_tool_completions(
    github: &GithubWrapper,
    document: &Document,
    replace_range: StdRange<usize>,
    substring: &str,
) -> Result<Vec<CompletionItem>> {
    let idx_slash = substring.find('/');
    let idx_at = idx_slash
        .and_then(|idx| substring[idx..].find('@'))
        .map(|idx| idx + idx_slash.unwrap());
    match (idx_slash, idx_at) {
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
    }
}

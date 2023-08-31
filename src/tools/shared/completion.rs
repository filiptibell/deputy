use semver::Version;
use tower_lsp::jsonrpc::Error;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::github::GithubErrorExt;
use crate::github::GithubWrapper;

use super::constants::*;

async fn complete_tool_author(
    _github: &GithubWrapper,
    author: &str,
) -> Result<Vec<CompletionItem>> {
    let author_low = author.to_ascii_lowercase();
    Ok(KNOWN_TOOLS
        .keys()
        .filter(|a| author_low.is_empty() || a.to_ascii_lowercase().starts_with(&author_low))
        .map(|a| CompletionItem {
            label: a.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            insert_text: Some(a.to_string()),
            ..Default::default()
        })
        .collect())
}

async fn complete_tool_name(
    _github: &GithubWrapper,
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
                insert_text: Some(repo.to_string()),
                ..Default::default()
            })
            .collect()),
    }
}

async fn complete_tool_version(
    github: &GithubWrapper,
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
                    // TODO: Fix appending '7.3.0' instead of replacing, resulting in versions like '7.3.7.3.0'
                    // We should probably add in TextEdit structs for all completions just to make sure
                    ..Default::default()
                })
                .collect())
        }
    }
}

pub async fn get_tool_completions(
    github: &GithubWrapper,
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
            complete_tool_version(github, author, name, version).await
        }
        (Some(idx_slash), _) => {
            let author = &substring[..idx_slash];
            let name = &substring[idx_slash + 1..];
            complete_tool_name(github, author, name).await
        }
        _ => complete_tool_author(github, substring).await,
    }
}

use semver::Version;
use tower_lsp::lsp_types::*;

use crate::github::GithubWrapper;

use super::actions::*;
use super::manifest::*;

pub fn diagnose_tool_spec(tool: &ManifestTool, range: &Range) -> Option<Diagnostic> {
    match tool.spec() {
        Ok(_) => None,
        Err(e) => Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: e.to_string(),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }),
    }
}

pub async fn diagnose_tool_version(
    github: &GithubWrapper,
    uri: &Url,
    tool: &ManifestTool,
    range: &Range,
) -> Option<Diagnostic> {
    let spec = tool.spec().ok()?;
    let releases = github
        .get_repository_releases(&spec.author, &spec.name)
        .await
        .ok()?;

    if releases.is_empty() {
        return Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: format!(
                "No releases were found for tool '{}/{}'",
                spec.author, spec.name
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }

    let latest_tag = releases[0].tag_name.trim_start_matches('v');
    let latest_version = Version::parse(latest_tag).ok()?;

    if latest_version > spec.version {
        let metadata = CodeActionMetadata::LatestVersion {
            source_uri: uri.clone(),
            source_text: tool.val_text.to_string(),
            version_current: spec.version.to_string(),
            version_latest: latest_version.to_string(),
        };
        Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: format!(
                "A newer tool version is available.\
                \nThe latest version is `{latest_version}`"
            ),
            severity: Some(DiagnosticSeverity::INFORMATION),
            data: Some(metadata.into()),
            ..Default::default()
        })
    } else {
        None
    }
}

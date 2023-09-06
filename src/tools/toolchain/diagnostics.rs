use semver::Version;
use tower_lsp::lsp_types::*;

use crate::clients::*;
use crate::util::*;

use super::super::util::*;
use super::manifest::*;
use super::tool_spec::*;

pub fn diagnose_tool_spec(tool: &ManifestTool, range: &Range) -> Option<Diagnostic> {
    match tool.spec() {
        Ok(_) => None,
        Err(e) => {
            // Missing author / name / version usually happens
            // when the user is typing, and is not really an error
            let severity = match e {
                ToolSpecError::MissingAuthor
                | ToolSpecError::MissingName
                | ToolSpecError::MissingVersion => DiagnosticSeverity::WARNING,
                _ => DiagnosticSeverity::ERROR,
            };
            Some(Diagnostic {
                source: Some(String::from("Tools")),
                range: *range,
                message: e.to_string(),
                severity: Some(severity),
                ..Default::default()
            })
        }
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
        .await;

    if releases.as_deref().is_err_and(|e| e.is_not_found_error()) {
        return Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: format!("No tool was found for '{}/{}'", spec.author, spec.name),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }

    let releases = releases.ok()?;
    if releases.is_empty() {
        return Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: format!("No releases were found for '{}/{}'", spec.author, spec.name),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }

    let spec_ver_name = spec.version.to_string();
    let spec_tag_name = format!("v{}", spec.version);
    if !releases
        .iter()
        .any(|r| r.tag_name == spec_ver_name || r.tag_name == spec_tag_name)
    {
        return Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: format!(
                "No release was found matching the version '{}'",
                spec.version
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
            source_text: tool.source().to_string(),
            version_current: spec.version.to_string(),
            version_latest: latest_version.to_string(),
        };
        return Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: format!(
                "A newer version is available.\
                \nThe latest version is `{latest_version}`"
            ),
            severity: Some(DiagnosticSeverity::INFORMATION),
            data: Some(
                ResolveContext {
                    uri: uri.clone(),
                    value: metadata,
                }
                .into(),
            ),
            ..Default::default()
        });
    }

    None
}

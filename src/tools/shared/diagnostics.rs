use semver::Version;
use tower_lsp::lsp_types::*;

use crate::github::GithubWrapper;

use super::actions::*;
use super::manifest::*;

pub fn diagnose_tool_spec(tool: &ManifestTool, range: &Range) -> Option<Diagnostic> {
    if let Err(err) = tool.spec() {
        Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: err.to_string(),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        })
    } else {
        None
    }
}

pub async fn diagnose_tool_version(
    github: &GithubWrapper,
    uri: &Url,
    tool: &ManifestTool,
    range: &Range,
) -> Option<Diagnostic> {
    let spec = match tool.spec() {
        Err(_) => return None,
        Ok(s) => s,
    };
    let latest = match github.get_latest_release(spec.author, spec.name).await {
        Err(_) => return None,
        Ok(l) => l,
    };

    let latest_tag = latest.tag_name.trim_start_matches('v');
    let latest_version = match Version::parse(latest_tag) {
        Err(_) => return None,
        Ok(v) => v,
    };

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

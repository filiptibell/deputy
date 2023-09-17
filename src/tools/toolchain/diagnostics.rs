use semver::Version;
use tower_lsp::lsp_types::*;

use crate::clients::github::models::RepositoryRelease;
use crate::clients::github::models::RepositoryReleaseAsset;
use crate::clients::*;

use super::super::util::*;
use super::compat::*;
use super::manifest::*;

pub fn diagnose_tool_spec(tool: &ManifestTool, range: &Range) -> Option<Diagnostic> {
    match tool.spec() {
        Ok(_) => None,
        Err(e) => Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: e.to_string(),
            severity: Some(e.diagnostic_severity()),
            ..Default::default()
        }),
    }
}

pub async fn diagnose_tool_version(
    clients: &Clients,
    uri: &Url,
    tool: &ManifestTool,
    range: &Range,
) -> Option<Diagnostic> {
    let spec = tool.spec().ok()?;
    let releases = clients
        .github
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

    let spec_ver_name = spec.tag.to_string();
    let spec_tag_name = format!("v{}", spec.tag);
    let matching_release = releases
        .iter()
        .find(|r| r.tag_name == spec_ver_name || r.tag_name == spec_tag_name);
    if matching_release.is_none() {
        return Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: format!("No release was found matching the tag '{}'", spec.tag),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }

    let has_compatible_asset = matching_release
        .map(is_release_compatible)
        .unwrap_or_default();
    if !has_compatible_asset {
        return Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: format!("No compatible asset was found for release '{}'", spec.tag),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }

    let latest_tag = releases[0].tag_name.trim_start_matches('v');
    let latest_version = Version::parse(latest_tag).ok()?;
    let current_version = Version::parse(&spec_ver_name).ok()?;
    if latest_version > current_version {
        let name = spec.name;
        let metadata = CodeActionMetadata::LatestVersion {
            source_uri: uri.clone(),
            source_text: tool.source().to_string(),
            version_current: spec.tag.to_string(),
            version_latest: latest_version.to_string(),
        };
        return Some(Diagnostic {
            source: Some(String::from("Tools")),
            range: *range,
            message: format!(
                "A newer version of `{name}` is available.\
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

fn is_release_compatible(release: &RepositoryRelease) -> bool {
    release.assets.iter().any(is_release_asset_compatible)
}

fn is_release_asset_compatible(release_asset: &RepositoryReleaseAsset) -> bool {
    release_asset
        .name
        .parse::<ArtifactCompat>()
        .ok()
        .map(|c| c.is_compatible())
        .unwrap_or_default()
}

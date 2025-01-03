use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::parser::Dependency;
use crate::util::{VersionReqExt, Versioned};

use super::super::shared::*;
use super::{Clients, Document};

pub async fn get_npm_diagnostics(
    clients: &Clients,
    doc: &Document,
    dep: &Dependency,
) -> Result<Vec<Diagnostic>> {
    let Some(dep_version) = dep.spec().and_then(|s| s.contents.version.as_ref()) else {
        return Ok(Vec::new());
    };
    let Ok(version_req) = dep.parse_version_req() else {
        return Ok(Vec::new());
    };
    let version = version_req.minimum_version();

    // Fetch versions and make sure there is at least one
    let meta = match clients
        .npm
        .get_registry_metadata(dep.name().unquoted())
        .await
    {
        Ok(v) => v,
        Err(e) => {
            if e.is_not_found_error() {
                return Ok(vec![Diagnostic {
                    source: Some(String::from("NPM")),
                    range: dep.name().range,
                    message: format!(
                        "No package exists with the name `{}`",
                        dep.name().unquoted()
                    ),
                    severity: Some(DiagnosticSeverity::ERROR),
                    ..Default::default()
                }]);
            } else {
                return Ok(Vec::new());
            }
        }
    };

    // Check if any version meeting the one specified exists
    if !meta.versions.iter().any(|(_, v)| {
        v.version
            .parse_version()
            .is_ok_and(|v| version_req.matches(&v))
    }) {
        return Ok(vec![Diagnostic {
            source: Some(String::from("NPM")),
            range: dep_version.range,
            message: format!(
                "Version `{version}` does not exist for the package `{}`",
                dep.name().unquoted()
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    // Everything is OK - but we may be able to suggest new versions...
    // ... try to find the latest non-prerelease version
    let Some(latest_version) = version.extract_latest_version(meta.versions.values().cloned())
    else {
        return Ok(Vec::new());
    };

    if !latest_version.is_semver_compatible {
        let latest_version_string = latest_version.item_version.to_string();

        let metadata = CodeActionMetadata::LatestVersion {
            edit_range: dep_version.range,
            source_uri: doc.uri().clone(),
            source_text: dep_version.quoted().to_string(),
            version_current: version.to_string(),
            version_latest: latest_version_string.to_string(),
        };

        return Ok(vec![Diagnostic {
            source: Some(String::from("NPM")),
            range: dep_version.range,
            message: format!(
                "A newer version of `{}` is available.\
                \nThe latest version is `{latest_version_string}`",
                dep.name().unquoted()
            ),
            severity: Some(DiagnosticSeverity::INFORMATION),
            data: Some(
                ResolveContext {
                    uri: doc.uri().clone(),
                    value: metadata,
                }
                .into(),
            ),
            ..Default::default()
        }]);
    }

    Ok(Vec::new())
}

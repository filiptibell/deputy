use semver::VersionReq;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::parser::SimpleDependency;
use crate::util::Versioned;

use super::super::shared::*;
use super::{Clients, Document, VersionReqExt};

pub async fn get_wally_diagnostics(
    clients: &Clients,
    doc: &Document,
    index_url: &str,
    tool: &SimpleDependency,
) -> Result<Vec<Diagnostic>> {
    let parsed = tool.parsed_spec();

    // Check for any missing fields
    let missing_author = parsed.author.unquoted().is_empty();
    let missing_name = parsed.name.as_ref().is_none_or(|r| r.unquoted().is_empty());
    let missing_version = parsed
        .version
        .as_ref()
        .is_none_or(|v| v.unquoted().is_empty());
    let missing_diag = if missing_author {
        Some("Missing package author")
    } else if missing_name {
        Some("Missing package name")
    } else if missing_version {
        Some("Missing package version")
    } else {
        None
    };

    // Propagate missing fields diagnostic, if any
    if let Some(diag) = missing_diag {
        return Ok(vec![Diagnostic {
            source: Some(String::from("Wally")),
            range: tool.spec.range,
            message: diag.to_string(),
            severity: Some(DiagnosticSeverity::WARNING), // Most likely during typing, don't emit a hard error
            ..Default::default()
        }]);
    }

    // Fetch versions and make sure there is at least one
    let parsed = parsed.into_full().expect("nothing was missing");
    let Ok(parsed_version_req) = VersionReq::parse(parsed.version.unquoted()) else {
        return Ok(Vec::new());
    };
    let parsed_version = parsed_version_req.minimum_version();

    let metadatas = match clients
        .wally
        .get_index_metadatas(index_url, parsed.author.unquoted(), parsed.name.unquoted())
        .await
    {
        Ok(v) => v,
        Err(e) => {
            if e.is_not_found_error() {
                return Ok(vec![Diagnostic {
                    source: Some(String::from("Wally")),
                    range: parsed.range(),
                    message: format!(
                        "No package exists with the name `{}/{}`",
                        parsed.author.unquoted(),
                        parsed.name.unquoted(),
                    ),
                    severity: Some(DiagnosticSeverity::ERROR),
                    ..Default::default()
                }]);
            } else {
                return Ok(Vec::new());
            }
        }
    };
    if metadatas.is_empty() {
        return Ok(vec![Diagnostic {
            source: Some(String::from("Wally")),
            range: parsed.range(),
            message: format!(
                "No versions exist for the package `{}/{}`",
                parsed.author.unquoted(),
                parsed.name.unquoted(),
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    // Check if any version meeting the one specified exists
    if !metadatas.iter().any(|release| {
        release
            .package
            .version
            .parse_version()
            .is_ok_and(|v| parsed_version_req.matches(&v))
    }) {
        return Ok(vec![Diagnostic {
            source: Some(String::from("Wally")),
            range: parsed.range(),
            message: format!(
                "Version `{parsed_version}` does not exist for the package `{}/{}`",
                parsed.author.unquoted(),
                parsed.name.unquoted(),
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    // Everything is OK - but we may be able to suggest new versions...
    // ... try to find the latest non-prerelease version
    let Some(latest_version) = parsed_version.extract_latest_version(metadatas) else {
        return Ok(Vec::new());
    };

    if !latest_version.is_semver_compatible {
        let latest_version_string = latest_version.item_version.to_string();

        let metadata = CodeActionMetadata::LatestVersion {
            edit_range: parsed.version.range,
            source_uri: doc.uri().clone(),
            source_text: parsed.version.quoted().to_string(),
            version_current: parsed_version.to_string(),
            version_latest: latest_version_string.to_string(),
        };

        return Ok(vec![Diagnostic {
            source: Some(String::from("Wally")),
            range: parsed.range(),
            message: format!(
                "A newer version of `{}/{}` is available.\
                \nThe latest version is `{latest_version_string}`",
                parsed.author.unquoted(),
                parsed.name.unquoted()
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

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::parser::SimpleDependency;
use crate::util::Versioned;

use super::super::shared::*;
use super::{Clients, Document, LspUriExt};

fn diag_source_for_doc(doc: &Document) -> String {
    if doc
        .uri()
        .file_name()
        .is_some_and(|n| n.eq_ignore_ascii_case("aftman.toml"))
    {
        String::from("Aftman")
    } else {
        String::from("Rokit")
    }
}

pub async fn get_rokit_diagnostics(
    clients: &Clients,
    doc: &Document,
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
        Some("Missing tool author")
    } else if missing_name {
        Some("Missing tool name")
    } else if missing_version {
        Some("Missing tool version")
    } else {
        None
    };

    // Propagate missing fields diagnostic, if any
    if let Some(diag) = missing_diag {
        return Ok(vec![Diagnostic {
            source: Some(diag_source_for_doc(doc)),
            range: tool.spec.range,
            message: diag.to_string(),
            severity: Some(DiagnosticSeverity::WARNING), // Most likely during typing, don't emit a hard error
            ..Default::default()
        }]);
    }

    // Fetch releases and make sure there is at least one
    let parsed = parsed.into_full().expect("nothing was missing");
    let parsed_version = parsed.version.unquoted().trim_start_matches('v');
    let releases = match clients
        .github
        .get_repository_releases(parsed.author.unquoted(), parsed.name.unquoted())
        .await
    {
        Ok(v) => v,
        Err(e) => {
            if e.is_not_found_error() {
                return Ok(vec![Diagnostic {
                    source: Some(diag_source_for_doc(doc)),
                    range: parsed.range(),
                    message: format!(
                        "No tool exists for `{}/{}`",
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
    if releases.is_empty() {
        return Ok(vec![Diagnostic {
            source: Some(diag_source_for_doc(doc)),
            range: parsed.range(),
            message: format!(
                "No releases exist for the tool `{}/{}`",
                parsed.author.unquoted(),
                parsed.name.unquoted(),
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    // Check if the exact version specified exists as a release
    if !releases.iter().any(|release| {
        release
            .tag_name
            .trim_start_matches('v')
            .eq_ignore_ascii_case(parsed_version)
    }) {
        return Ok(vec![Diagnostic {
            source: Some(diag_source_for_doc(doc)),
            range: parsed.range(),
            message: format!(
                "Version `{parsed_version}` does not exist for the tool `{}/{}`",
                parsed.author.unquoted(),
                parsed.name.unquoted(),
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    // Everything is OK - but we may be able to suggest new versions...
    // ... try to find the latest non-prerelease version
    let Some(latest_version) = parsed_version.extract_latest_version(releases) else {
        return Ok(Vec::new());
    };

    if !latest_version.is_exactly_compatible {
        let latest_version_string = latest_version.item_version.to_string();

        let metadata = CodeActionMetadata::LatestVersion {
            edit_range: parsed.version.range,
            source_uri: doc.uri().clone(),
            source_text: parsed.version.quoted().to_string(),
            version_current: parsed_version.to_string(),
            version_latest: latest_version_string.to_string(),
        };

        return Ok(vec![Diagnostic {
            source: Some(diag_source_for_doc(doc)),
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

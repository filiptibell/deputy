use async_language_server::{
    lsp_types::{Diagnostic, DiagnosticSeverity},
    server::{Document, ServerResult},
    tree_sitter_utils::ts_range_to_lsp_range,
};
use tree_sitter::Node;

use crate::parser::rokit;
use crate::util::Versioned;

use super::super::shared::*;
use super::Clients;

fn diag_source_for_doc(doc: &Document) -> String {
    if doc
        .url()
        .to_file_path()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
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
    node: Node<'_>,
) -> ServerResult<Vec<Diagnostic>> {
    let Some(dep) = rokit::parse_dependency(node) else {
        return Ok(Vec::new());
    };

    // Check for any missing fields
    let ranges = dep.spec_ranges(doc);
    let missing_diag = if ranges.owner.is_none() {
        Some("Missing tool author")
    } else if ranges.repository.is_none() {
        Some("Missing tool name")
    } else if ranges.version.is_none() {
        Some("Missing tool version")
    } else {
        None
    };

    // Propagate missing fields diagnostic, if any
    if let Some(diag) = missing_diag {
        return Ok(vec![Diagnostic {
            source: Some(diag_source_for_doc(doc)),
            range: ts_range_to_lsp_range(dep.spec.range()),
            message: diag.to_string(),
            severity: Some(DiagnosticSeverity::WARNING), // Most likely during typing, don't emit a hard error
            ..Default::default()
        }]);
    }

    let (Some(owner), Some(repository), Some(version)) = ranges.text(doc) else {
        return Ok(Vec::new());
    };

    // Fetch releases and make sure there is at least one
    let parsed_version = version.trim_start_matches('v');
    let releases = match clients
        .github
        .get_repository_releases(owner, repository)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            if e.is_not_found_error() {
                return Ok(vec![Diagnostic {
                    source: Some(diag_source_for_doc(doc)),
                    range: ts_range_to_lsp_range(dep.spec.range()),
                    message: format!("No tool exists for `{owner}/{repository}`"),
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
            range: ts_range_to_lsp_range(dep.spec.range()),
            message: format!("No releases exist for the tool `{owner}/{repository}`"),
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
            range: ts_range_to_lsp_range(dep.spec.range()),
            message: format!(
                "Version `{parsed_version}` does not exist for the tool `{owner}/{repository}`"
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
            edit_range: ts_range_to_lsp_range(ranges.version.unwrap()),
            source_uri: doc.url().clone(),
            source_text: version.to_string(),
            version_current: parsed_version.to_string(),
            version_latest: latest_version_string.to_string(),
        };

        return Ok(vec![Diagnostic {
            source: Some(diag_source_for_doc(doc)),
            range: ts_range_to_lsp_range(dep.spec.range()),
            message: format!(
                "A newer version of `{}/{}` is available.\
                \nThe latest version is `{latest_version_string}`",
                owner, repository
            ),
            severity: Some(DiagnosticSeverity::INFORMATION),
            data: Some(
                ResolveContext {
                    uri: doc.url().clone(),
                    value: metadata,
                }
                .into(),
            ),
            ..Default::default()
        }]);
    }

    Ok(Vec::new())
}

use semver::VersionReq;

use async_language_server::{
    lsp_types::{Diagnostic, DiagnosticSeverity},
    server::{Document, ServerResult},
    tree_sitter_utils::ts_range_to_lsp_range,
};
use tree_sitter::Node;

use crate::parser::wally;
use crate::util::{VersionReqExt, Versioned};

use super::super::shared::*;
use super::Clients;

pub async fn get_wally_diagnostics(
    clients: &Clients,
    doc: &Document,
    index_url: &str,
    node: Node<'_>,
) -> ServerResult<Vec<Diagnostic>> {
    let Some(dep) = wally::parse_dependency(node) else {
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
            source: Some(String::from("Wally")),
            range: ts_range_to_lsp_range(dep.spec.range()),
            message: diag.to_string(),
            severity: Some(DiagnosticSeverity::WARNING), // Most likely during typing, don't emit a hard error
            ..Default::default()
        }]);
    }

    let (Some(owner), Some(repository), Some(version)) = ranges.text(doc) else {
        return Ok(Vec::new());
    };

    // Fetch versions and make sure there is at least one
    let Ok(parsed_version_req) = VersionReq::parse(version) else {
        return Ok(Vec::new());
    };
    let parsed_version = parsed_version_req.minimum_version();

    let metadatas = match clients
        .wally
        .get_index_metadatas(index_url, owner, repository)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            if e.is_not_found_error() {
                return Ok(vec![Diagnostic {
                    source: Some(String::from("Wally")),
                    range: ts_range_to_lsp_range(dep.spec.range()),
                    message: format!("No package exists with the name `{owner}/{repository}`"),
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
            range: ts_range_to_lsp_range(dep.spec.range()),
            message: format!("No versions exist for the package `{owner}/{repository}`"),
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
            range: ts_range_to_lsp_range(dep.spec.range()),
            message: format!(
                "Version `{parsed_version}` does not exist for the package `{owner}/{repository}`"
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
            edit_range: ts_range_to_lsp_range(ranges.version.unwrap()),
            source_uri: doc.url().clone(),
            source_text: version.to_string(),
            version_current: parsed_version.to_string(),
            version_latest: latest_version_string.to_string(),
        };

        return Ok(vec![Diagnostic {
            source: Some(String::from("Wally")),
            range: ts_range_to_lsp_range(dep.spec.range()),
            message: format!(
                "A newer version of `{owner}/{repository}` is available.\
                \nThe latest version is `{latest_version_string}`",
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

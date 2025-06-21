use async_language_server::{
    lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag},
    server::{Document, ServerResult},
    tree_sitter::Node,
    tree_sitter_utils::ts_range_to_lsp_range,
};

use crate::parser::npm;
use crate::util::{VersionReqExt, Versioned};

use super::super::shared::*;
use super::Clients;

pub async fn get_npm_diagnostics(
    clients: &Clients,
    doc: &Document,
    node: Node<'_>,
) -> ServerResult<Vec<Diagnostic>> {
    let Some(dep) = npm::parse_dependency(node) else {
        return Ok(Vec::new());
    };

    let (name, spec) = dep.text(doc);
    if spec.starts_with("file:") || spec.starts_with("github:") || spec.starts_with("git+") {
        return Ok(Vec::new()); // Ignore these spec formats, for now
    }

    let Ok(version_req) = spec.parse_version_req() else {
        return Ok(Vec::new());
    };
    let version = version_req.minimum_version();

    // Fetch versions and make sure there is at least one
    let meta = match clients.npm.get_registry_metadata(&name).await {
        Ok(v) => v,
        Err(e) => {
            if e.is_not_found_error() {
                return Ok(vec![Diagnostic {
                    source: Some(String::from("NPM")),
                    range: ts_range_to_lsp_range(dep.name.range()),
                    message: format!("No package exists with the name `{}`", name),
                    severity: Some(DiagnosticSeverity::ERROR),
                    ..Default::default()
                }]);
            } else {
                return Ok(Vec::new());
            }
        }
    };

    let mut has_versions = false;
    let mut deprecation_reason = None;
    for version in meta.versions.values().filter(|v| {
        v.version
            .parse_version()
            .is_ok_and(|v| version_req.matches(&v))
    }) {
        has_versions = true;
        let Some(reason) = version.deprecated.as_deref() else {
            deprecation_reason = None;
            break;
        };

        if deprecation_reason.is_none() {
            deprecation_reason = Some(reason);
        }
    }

    // Check if any version meeting the one specified exists
    if !has_versions {
        return Ok(vec![Diagnostic {
            source: Some(String::from("NPM")),
            range: ts_range_to_lsp_range(dep.spec.range()),
            message: format!("Version `{version}` does not exist for the package `{name}`"),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    if let Some(deprecation_reason) = deprecation_reason {
        return Ok(vec![Diagnostic {
            source: Some(String::from("NPM")),
            range: ts_range_to_lsp_range(dep.spec.range()),
            message: format!("Version `{version}` is deprecated: {deprecation_reason}"),
            severity: Some(DiagnosticSeverity::WARNING),
            tags: Some(vec![DiagnosticTag::DEPRECATED]),
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
            edit_range: ts_range_to_lsp_range(dep.spec.range()),
            source_uri: doc.url().clone(),
            source_text: spec.to_string(),
            version_current: version.to_string(),
            version_latest: latest_version_string.to_string(),
        };

        return Ok(vec![Diagnostic {
            source: Some(String::from("NPM")),
            range: ts_range_to_lsp_range(dep.spec.range()),
            message: format!(
                "A newer version of `{name}` is available.\
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

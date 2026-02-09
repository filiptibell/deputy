use async_language_server::{
    lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag},
    server::{Document, ServerResult},
    tree_sitter::Node,
    tree_sitter_utils::ts_range_to_lsp_range,
};

use deputy_parser::pyproject::{self, PyProjectDependency};
use deputy_versioning::{PepVersionReqExt, PepVersioned};

use crate::shared::{CodeActionMetadata, ResolveContext, did_you_mean};

use super::Clients;

pub async fn get_pyproject_diagnostics(
    clients: &Clients,
    doc: &Document,
    node: Node<'_>,
) -> ServerResult<Vec<Diagnostic>> {
    let Some(dep) = pyproject::parse_dependency(node) else {
        return Ok(Vec::new());
    };

    // Check for any missing fields
    let ranges = dep.spec_ranges(doc);
    if ranges.name.is_none() {
        return Ok(vec![Diagnostic {
            source: Some(String::from("PyPI")),
            range: ts_range_to_lsp_range(dep.spec_node.range()),
            message: String::from("Missing package name"),
            severity: Some(DiagnosticSeverity::WARNING),
            ..Default::default()
        }]);
    }

    let (Some(name), version) = dep.text(doc) else {
        return Ok(Vec::new());
    };

    // No version spec is valid PEP 508 (e.g. just "requests")
    let Some(version) = version else {
        return Ok(Vec::new());
    };

    let Ok(version_req) = version.parse_version_req() else {
        return Ok(Vec::new());
    };
    let version_min = version_req.minimum_version();

    // Fetch versions and make sure there is at least one
    let meta = match clients.pypi.get_simple_metadata(&name).await {
        Ok(v) => v,
        Err(e) => {
            if e.is_not_found_error() {
                return Ok(vec![Diagnostic {
                    source: Some(String::from("PyPI")),
                    range: ts_range_to_lsp_range(ranges.name.unwrap()),
                    message: format!("No package exists with the name `{name}`"),
                    severity: Some(DiagnosticSeverity::ERROR),
                    ..Default::default()
                }]);
            }
            return Ok(Vec::new());
        }
    };

    let versions = meta.versions();

    // Check if any version meeting the requirement exists
    let mut has_versions = false;
    let mut all_yanked = true;
    for v in &versions {
        if v.parse_version()
            .is_ok_and(|parsed| version_req.matches(&parsed))
        {
            has_versions = true;
            if !v.yanked {
                all_yanked = false;
                break;
            }
        }
    }

    if !has_versions {
        return Ok(vec![Diagnostic {
            source: Some(String::from("PyPI")),
            range: ts_range_to_lsp_range(ranges.version.unwrap()),
            message: format!("Version `{version_min}` does not exist for the package `{name}`"),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    if has_versions && all_yanked {
        return Ok(vec![Diagnostic {
            source: Some(String::from("PyPI")),
            range: ts_range_to_lsp_range(ranges.version.unwrap()),
            message: format!("Version `{version_min}` is yanked for the package `{name}`"),
            severity: Some(DiagnosticSeverity::WARNING),
            tags: Some(vec![DiagnosticTag::DEPRECATED]),
            ..Default::default()
        }]);
    }

    // Version is valid - collect remaining diagnostics
    let mut diagnostics = Vec::new();

    // Check for newer versions available
    if let Some(latest_version) = version_min.extract_latest_version(versions)
        && !latest_version.is_compatible
    {
        let latest_version_string = latest_version.item_version.to_string();

        let metadata = CodeActionMetadata::LatestVersion {
            edit_range: ts_range_to_lsp_range(ranges.version.unwrap()),
            source_uri: doc.url().clone(),
            source_text: version.clone(),
            version_current: version_min.to_string(),
            version_latest: latest_version_string.clone(),
        };

        diagnostics.push(Diagnostic {
            source: Some(String::from("PyPI")),
            range: ts_range_to_lsp_range(dep.spec_node.range()),
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
        });
    }

    // Check extras against known extras from the registry
    if let Ok(meta) = clients.pypi.get_registry_metadata(&name).await
        && let Some(known_extras) = &meta.info.provides_extra
    {
        diagnostics.extend(get_pyproject_diagnostics_extras(doc, &dep, known_extras));
    }

    Ok(diagnostics)
}

fn get_pyproject_diagnostics_extras(
    doc: &Document,
    dep: &PyProjectDependency<'_>,
    known_extras: &[String],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let txt = doc.text();

    for extra_range in dep.extra_ranges(doc) {
        let extra = txt
            .byte_slice(extra_range.start_byte..extra_range.end_byte)
            .as_str()
            .unwrap_or_default()
            .trim();

        if !known_extras.iter().any(|e| e.eq_ignore_ascii_case(extra)) {
            diagnostics.push(Diagnostic {
                source: Some(String::from("PyPI")),
                range: ts_range_to_lsp_range(extra_range),
                message: match did_you_mean(extra, known_extras) {
                    Some(suggestion) => {
                        format!("Unknown extra `{extra}` - did you mean `{suggestion}`?")
                    }
                    None => format!("Unknown extra `{extra}`"),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                ..Default::default()
            });
        }
    }

    diagnostics
}

use async_language_server::{
    lsp_types::{Diagnostic, DiagnosticSeverity},
    server::{Document, ServerResult},
    tree_sitter::Node,
    tree_sitter_utils::ts_range_to_lsp_range,
};

use deputy_parser::gomod;
use deputy_versioning::Versioned;

use crate::shared::{CodeActionMetadata, ResolveContext};

use super::Clients;

pub async fn get_gomod_diagnostics(
    clients: &Clients,
    doc: &Document,
    node: Node<'_>,
) -> ServerResult<Vec<Diagnostic>> {
    let Some(dep) = gomod::parse_dependency(node) else {
        return Ok(Vec::new());
    };

    let (path, version) = dep.text(doc);
    let Some(version) = version else {
        return Ok(Vec::new()); // Incomplete spec, skip
    };

    let parsed_version = version.trim_start_matches('v');

    // Fetch versions from the Go proxy
    let versions = match clients.golang.get_module_versions(&path).await {
        Ok(v) => v,
        Err(e) => {
            if e.is_not_found_error() {
                return Ok(vec![Diagnostic {
                    source: Some(String::from("Go")),
                    range: ts_range_to_lsp_range(dep.path.range()),
                    message: format!("No published module exists for `{path}`"),
                    severity: Some(DiagnosticSeverity::ERROR),
                    ..Default::default()
                }]);
            }
            return Ok(Vec::new());
        }
    };

    if versions.is_empty() {
        return Ok(vec![Diagnostic {
            source: Some(String::from("Go")),
            range: ts_range_to_lsp_range(dep.path.range()),
            message: format!("No versions exist for the module `{path}`"),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    // Check if the exact version specified exists
    if !versions.iter().any(|v| {
        v.trim_start_matches('v')
            .eq_ignore_ascii_case(parsed_version)
    }) {
        return Ok(vec![Diagnostic {
            source: Some(String::from("Go")),
            range: ts_range_to_lsp_range(dep.version.unwrap().range()),
            message: format!("Version `{version}` does not exist for the module `{path}`"),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    // Everything is OK - but we may be able to suggest new versions...
    // ... try to find the latest non-prerelease version
    let stripped_versions: Vec<String> = versions
        .iter()
        .map(|v| v.trim_start_matches('v').to_string())
        .collect();

    let Some(latest_version) = parsed_version.extract_latest_version(stripped_versions) else {
        return Ok(Vec::new());
    };

    if !latest_version.is_exactly_compatible {
        let latest_version_string = format!("v{}", latest_version.item_version);

        let version_node = dep.version.unwrap();
        let metadata = CodeActionMetadata::LatestVersion {
            edit_range: ts_range_to_lsp_range(version_node.range()),
            source_uri: doc.url().clone(),
            source_text: version.clone(),
            version_current: parsed_version.to_string(),
            version_latest: latest_version.item_version.to_string(),
        };

        return Ok(vec![Diagnostic {
            source: Some(String::from("Go")),
            range: ts_range_to_lsp_range(node.range()),
            message: format!(
                "A newer version of `{path}` is available.\
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

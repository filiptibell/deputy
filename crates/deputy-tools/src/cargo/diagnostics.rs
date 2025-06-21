use tracing::debug;

use async_language_server::{
    lsp_types::{Diagnostic, DiagnosticSeverity},
    server::{Document, ServerResult},
    text_utils::RangeExt,
    tree_sitter::Node,
    tree_sitter_utils::ts_range_to_lsp_range,
};

use deputy_versioning::{VersionReq, VersionReqExt, Versioned};

use deputy_clients::crates::models::IndexMetadata;
use deputy_parser::{
    cargo::{self, CargoDependency},
    utils::unquote,
};

use crate::shared::{CodeActionMetadata, ResolveContext, did_you_mean};

use super::Clients;
use super::util::get_features;

pub async fn get_cargo_diagnostics(
    clients: &Clients,
    doc: &Document,
    node: Node<'_>,
) -> ServerResult<Vec<Diagnostic>> {
    let Some(dep) = cargo::parse_dependency(doc, node) else {
        return Ok(Vec::new());
    };

    let (name, _version) = dep.text(doc);

    let metas = match clients.crates.get_sparse_index_crate_metadatas(&name).await {
        Ok(v) => v,
        Err(e) => {
            if e.is_not_found_error() {
                return Ok(vec![Diagnostic {
                    source: Some(String::from("Cargo")),
                    range: ts_range_to_lsp_range(dep.name.range()),
                    message: format!("No package exists with the name `{name}`"),
                    severity: Some(DiagnosticSeverity::ERROR),
                    ..Default::default()
                }]);
            }
            return Ok(Vec::new());
        }
    };

    let mut diagnostics = Vec::new();
    diagnostics.extend(get_cargo_diagnostics_version(clients, doc, &dep, &metas)?);
    diagnostics.extend(get_cargo_diagnostics_features(clients, doc, &dep, &metas).await?);
    Ok(diagnostics)
}

fn get_cargo_diagnostics_version(
    _clients: &Clients,
    doc: &Document,
    dep: &CargoDependency<'_>,
    metas: &[IndexMetadata],
) -> ServerResult<Vec<Diagnostic>> {
    let (name, version) = dep.text(doc);

    let Ok(version_req) = VersionReq::parse(&version) else {
        return Ok(Vec::new());
    };
    let version_min = version_req.minimum_version();

    // Check if the specified package version exists in the index
    if !metas.iter().any(|r| {
        r.parse_version()
            .map(|version| version_req.matches(&version))
            .ok()
            .unwrap_or_default()
    }) {
        return Ok(vec![Diagnostic {
            source: Some(String::from("Cargo")),
            range: ts_range_to_lsp_range(dep.version.range()),
            message: format!("No version exists that matches requirement `{version}`"),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    // Try to find the latest non-prerelease version, filtering out
    // any version that has been yanked - unless we exactly specify it
    let latest_name = name.to_string();
    let Some(latest_version) = version_min
        .extract_latest_version_filtered(metas.iter().cloned(), |v| {
            !v.item.yanked || v.is_exactly_compatible
        })
    else {
        debug!("Failed to get latest crates.io version for '{latest_name}'");
        return Ok(Vec::new());
    };

    if !latest_version.is_semver_compatible {
        let latest_version_string = latest_version.item_version.to_string();

        let metadata = CodeActionMetadata::LatestVersion {
            edit_range: ts_range_to_lsp_range(dep.version.range().shrink(1, 1)),
            source_uri: doc.url().clone(),
            source_text: version.to_string(),
            version_current: version_min.to_string(),
            version_latest: latest_version_string.to_string(),
        };

        return Ok(vec![Diagnostic {
            source: Some(String::from("Cargo")),
            range: ts_range_to_lsp_range(dep.version.range()),
            message: format!(
                "A newer version of `{latest_name}` is available.\
                \nThe latest version is `{latest_version_string}`"
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

async fn get_cargo_diagnostics_features(
    clients: &Clients,
    doc: &Document,
    dep: &CargoDependency<'_>,
    _metas: &[IndexMetadata],
) -> ServerResult<Vec<Diagnostic>> {
    let (name, version) = dep.text(doc);

    let Some(known_features) = get_features(clients, &name, &version).await else {
        return Ok(Vec::new());
    };

    let mut diagnostics = Vec::new();
    for feat_node in dep.feature_nodes() {
        let feat = unquote(doc.node_text(feat_node));
        if !known_features.contains(&feat.to_string()) {
            diagnostics.push(Diagnostic {
                source: Some(String::from("Cargo")),
                range: ts_range_to_lsp_range(feat_node.range()),
                message: match did_you_mean(&feat, known_features.as_slice()) {
                    Some(suggestion) => {
                        format!("Unknown feature `{feat}` - did you mean `{suggestion}`?")
                    }
                    None => format!("Unknown feature `{feat}`"),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                ..Default::default()
            });
        }
    }

    Ok(diagnostics)
}

use async_language_server::{
    lsp_types::{Diagnostic, DiagnosticSeverity},
    server::{Document, ServerResult},
    text_utils::RangeExt,
    tree_sitter::Node,
    tree_sitter_utils::ts_range_to_lsp_range,
};
use tracing::debug;

use deputy_clients::crates::models::{IndexMetadata, LocalMetadata, WorkspaceDependencyMetadata};
use deputy_parser::{
    cargo::{self, CargoDependency},
    utils::unquote,
};
use deputy_versioning::{VersionReq, VersionReqExt, Versioned};

use crate::shared::{CodeActionMetadata, ResolveContext, did_you_mean};

use super::Clients;
use super::util::{
    LocalDependencyResolution, WorkspaceDependencyResolution, get_features,
    get_workspace_local_metadata, resolve_local_dependency, resolve_workspace_dependency,
};

pub async fn get_cargo_diagnostics(
    clients: &Clients,
    doc: &Document,
    node: Node<'_>,
) -> ServerResult<Vec<Diagnostic>> {
    let Some(dep) = cargo::parse_dependency(doc, node) else {
        return Ok(Vec::new());
    };
    let workspace_meta = match resolve_workspace_dependency(clients, doc, &dep).await {
        WorkspaceDependencyResolution::Resolved(metadata) => Some(metadata),
        WorkspaceDependencyResolution::Missing { name } => {
            return Ok(vec![get_cargo_diagnostics_missing_workspace_dependency(
                &dep, &name,
            )]);
        }
        WorkspaceDependencyResolution::NotWorkspace
        | WorkspaceDependencyResolution::Unavailable => None,
    };

    // For path dependencies, check version and features
    // against the local crate instead of the crates.io registry
    if let Some(path) = dep.path_text(doc) {
        let local_meta = match resolve_local_dependency(clients, doc.url(), &path).await {
            LocalDependencyResolution::Resolved(metadata) => metadata,
            LocalDependencyResolution::MissingPath => {
                return Ok(vec![get_cargo_diagnostics_missing_path_dependency(
                    &dep, &path,
                )]);
            }
            LocalDependencyResolution::MissingManifest => {
                return Ok(vec![get_cargo_diagnostics_missing_path_manifest(
                    &dep, &path,
                )]);
            }
            LocalDependencyResolution::Unavailable => return Ok(Vec::new()),
        };
        let mut diagnostics = Vec::new();
        diagnostics.extend(get_cargo_diagnostics_local_version(doc, &dep, &local_meta));
        diagnostics.extend(get_cargo_diagnostics_features(
            doc,
            &dep,
            &local_meta.features,
        ));
        return Ok(diagnostics);
    }

    if let Some(workspace_meta) = workspace_meta.as_ref() {
        if workspace_meta.is_path() {
            let Some(local_meta) = get_workspace_local_metadata(clients, workspace_meta).await
            else {
                return Ok(Vec::new());
            };
            let mut diagnostics = Vec::new();
            diagnostics.extend(get_cargo_diagnostics_local_version(doc, &dep, &local_meta));
            diagnostics.extend(get_cargo_diagnostics_features(
                doc,
                &dep,
                &local_meta.features,
            ));
            return Ok(diagnostics);
        }
    } else if dep.is_workspace() {
        return Ok(Vec::new());
    }

    // Git dependencies are not checked against crates.io
    // FUTURE: Implement proper resolution for git dependencies?
    if dep.git_text(doc).is_some()
        || workspace_meta
            .as_ref()
            .is_some_and(WorkspaceDependencyMetadata::is_git)
    {
        return Ok(Vec::new());
    }

    let (name, version) = dep.text(doc);
    let name = workspace_meta
        .as_ref()
        .map_or(name, |metadata| metadata.name.clone());
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

    let version = version.or_else(|| workspace_meta.map(|metadata| metadata.req));
    let Some(version) = version else {
        return Ok(Vec::new());
    };

    let mut diagnostics = Vec::new();
    if dep.version.is_some() {
        diagnostics.extend(get_cargo_diagnostics_version(doc, &dep, &version, &metas));
    }
    if let Some(known_features) = get_features(clients, &name, &version).await {
        diagnostics.extend(get_cargo_diagnostics_features(doc, &dep, &known_features));
    }
    Ok(diagnostics)
}

fn get_cargo_diagnostics_missing_workspace_dependency(
    dep: &CargoDependency<'_>,
    name: &str,
) -> Diagnostic {
    Diagnostic {
        source: Some(String::from("Cargo")),
        range: ts_range_to_lsp_range(dep.name.range()),
        message: format!("No workspace dependency exists with the name `{name}`"),
        severity: Some(DiagnosticSeverity::ERROR),
        ..Default::default()
    }
}

fn get_cargo_diagnostics_missing_path_dependency(
    dep: &CargoDependency<'_>,
    path: &str,
) -> Diagnostic {
    Diagnostic {
        source: Some(String::from("Cargo")),
        range: path_diagnostic_range(dep),
        message: format!("No local dependency exists at path `{path}`"),
        severity: Some(DiagnosticSeverity::ERROR),
        ..Default::default()
    }
}

fn get_cargo_diagnostics_missing_path_manifest(
    dep: &CargoDependency<'_>,
    path: &str,
) -> Diagnostic {
    Diagnostic {
        source: Some(String::from("Cargo")),
        range: path_diagnostic_range(dep),
        message: format!("No Cargo.toml exists for local dependency at path `{path}`"),
        severity: Some(DiagnosticSeverity::ERROR),
        ..Default::default()
    }
}

fn path_diagnostic_range(dep: &CargoDependency<'_>) -> async_language_server::lsp_types::Range {
    let path = dep.path.expect("path node must exist");
    ts_range_to_lsp_range(path.range())
}

fn get_cargo_diagnostics_local_version(
    doc: &Document,
    dep: &CargoDependency<'_>,
    local_meta: &LocalMetadata,
) -> Vec<Diagnostic> {
    let (_name, version) = dep.text(doc);
    let Some(version) = version else {
        return Vec::new();
    };

    let Ok(version_req) = VersionReq::parse(&version) else {
        return Vec::new();
    };

    let Some(local_version) = &local_meta.version else {
        return Vec::new();
    };

    if !version_req.matches(local_version) {
        let version_node = dep.version.expect("version node must exist");
        return vec![Diagnostic {
            source: Some(String::from("Cargo")),
            range: ts_range_to_lsp_range(version_node.range()),
            message: format!("No local version exists that matches requirement `{version}`"),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }];
    }

    Vec::new()
}

fn get_cargo_diagnostics_version(
    doc: &Document,
    dep: &CargoDependency<'_>,
    version: &str,
    metas: &[IndexMetadata],
) -> Vec<Diagnostic> {
    let (name, _) = dep.text(doc);
    let version_node = dep.version.expect("version node must exist");

    let Ok(version_req) = VersionReq::parse(version) else {
        return Vec::new();
    };
    let version_min = version_req.minimum_version();

    // Check if the specified package version exists in the index
    if !metas.iter().any(|r| {
        r.parse_version()
            .map(|version| version_req.matches(&version))
            .ok()
            .unwrap_or_default()
    }) {
        return vec![Diagnostic {
            source: Some(String::from("Cargo")),
            range: ts_range_to_lsp_range(version_node.range()),
            message: format!("No version exists that matches requirement `{version}`"),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }];
    }

    // Try to find the latest non-prerelease version, filtering out
    // any version that has been yanked - unless we exactly specify it
    let latest_name = name.clone();
    let Some(latest_version) = version_min
        .extract_latest_version_filtered(metas.iter().cloned(), |v| {
            !v.item.yanked || v.is_exactly_compatible
        })
    else {
        debug!("Failed to get latest crates.io version for '{latest_name}'");
        return Vec::new();
    };

    if !latest_version.is_semver_compatible {
        let latest_version_string = latest_version.item_version.to_string();

        let metadata = CodeActionMetadata::LatestVersion {
            edit_range: ts_range_to_lsp_range(version_node.range().shrink(1, 1)),
            source_uri: doc.url().clone(),
            source_text: version.to_string(),
            version_current: version_min.to_string(),
            version_latest: latest_version_string.clone(),
        };

        return vec![Diagnostic {
            source: Some(String::from("Cargo")),
            range: ts_range_to_lsp_range(version_node.range()),
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
        }];
    }

    Vec::new()
}

fn get_cargo_diagnostics_features(
    doc: &Document,
    dep: &CargoDependency<'_>,
    known_features: &[String],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for feat_node in dep.feature_nodes() {
        let feat = unquote(doc.node_text(feat_node));
        if !known_features.contains(&feat) {
            diagnostics.push(Diagnostic {
                source: Some(String::from("Cargo")),
                range: ts_range_to_lsp_range(feat_node.range()),
                message: match did_you_mean(&feat, known_features) {
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
    diagnostics
}

use semver::VersionReq;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tracing::debug;

use crate::parser::Dependency;
use crate::util::{VersionReqExt, Versioned};

use super::super::shared::*;
use super::crates::models::IndexMetadata;
use super::util::get_features;
use super::{Clients, Document};

// TODO: Enable feature diagnostics when we have a way to
// actually fetch *all* features from the index or the api
const SHOW_FEATURE_DIAGNOSTICS: bool = false;

pub async fn get_cargo_diagnostics(
    clients: &Clients,
    doc: &Document,
    dep: &Dependency,
) -> Result<Vec<Diagnostic>> {
    let metas = match clients
        .crates
        .get_sparse_index_crate_metadatas(dep.name().unquoted())
        .await
    {
        Ok(v) => v,
        Err(e) => {
            if e.is_not_found_error() {
                return Ok(vec![Diagnostic {
                    source: Some(String::from("Cargo")),
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

    let mut diagnostics = Vec::new();
    diagnostics.extend(get_cargo_diagnostics_version(clients, doc, dep, &metas).await?);
    diagnostics.extend(get_cargo_diagnostics_features(clients, doc, dep, &metas).await?);
    Ok(diagnostics)
}

async fn get_cargo_diagnostics_version(
    _clients: &Clients,
    doc: &Document,
    dep: &Dependency,
    metas: &[IndexMetadata],
) -> Result<Vec<Diagnostic>> {
    let Some(spec_version) = dep.spec().and_then(|s| s.contents.version.as_ref()) else {
        return Ok(Vec::new());
    };
    let Ok(version_req) = VersionReq::parse(spec_version.unquoted()) else {
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
            range: spec_version.range,
            message: format!(
                "No version exists that matches requirement `{}`",
                spec_version.unquoted()
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    // Try to find the latest non-prerelease version
    let latest_name = dep.name().unquoted().to_string();
    let Some(latest_version) = version_min.extract_latest_version(metas.iter().cloned()) else {
        debug!("Failed to get latest crates.io version for '{latest_name}'");
        return Ok(Vec::new());
    };

    if !latest_version.is_semver_compatible {
        let latest_version_string = latest_version.item_version.to_string();

        let metadata = CodeActionMetadata::LatestVersion {
            edit_range: spec_version.range,
            source_uri: doc.uri().clone(),
            source_text: spec_version.quoted().to_string(),
            version_current: version_min.to_string(),
            version_latest: latest_version_string.to_string(),
        };

        return Ok(vec![Diagnostic {
            source: Some(String::from("Cargo")),
            range: spec_version.range,
            message: format!(
                "A newer version of `{latest_name}` is available.\
                \nThe latest version is `{latest_version_string}`"
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

async fn get_cargo_diagnostics_features(
    clients: &Clients,
    _doc: &Document,
    dep: &Dependency,
    _metas: &[IndexMetadata],
) -> Result<Vec<Diagnostic>> {
    let Some(features) = dep.spec().and_then(|s| s.contents.features.as_ref()) else {
        return Ok(Vec::new());
    };

    if !SHOW_FEATURE_DIAGNOSTICS {
        return Ok(Vec::new());
    }

    let known_features = get_features(clients, dep).await;

    let mut diagnostics = Vec::new();
    for feat in features.contents.iter() {
        if !known_features.contains(&feat.unquoted().to_string()) {
            diagnostics.push(Diagnostic {
                source: Some(String::from("Cargo")),
                range: feat.range,
                message: match did_you_mean(feat.unquoted(), known_features.as_slice()) {
                    Some(suggestion) => {
                        format!(
                            "Unknown feature `{}` - did you mean `{suggestion}`?",
                            feat.unquoted()
                        )
                    }
                    None => format!("Unknown feature `{}`", feat.unquoted()),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                ..Default::default()
            });
        }
    }

    Ok(diagnostics)
}

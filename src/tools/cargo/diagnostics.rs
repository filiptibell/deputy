use semver::{Version, VersionReq};

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::parser::Dependency;

use super::super::shared::*;
use super::crates::models::IndexMetadata;
use super::util::{did_you_mean, get_features};
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
    let Some(version) = dep.spec().and_then(|s| s.contents.version.as_ref()) else {
        return Ok(Vec::new());
    };
    let Ok(version_req) = VersionReq::parse(version.unquoted()) else {
        return Ok(Vec::new());
    };

    // Check if the specified package version exists in the index
    if !metas.iter().any(|r| {
        Version::parse(&r.version)
            .map(|version| version_req.matches(&version))
            .ok()
            .unwrap_or_default()
    }) {
        return Ok(vec![Diagnostic {
            source: Some(String::from("Cargo")),
            range: version.range,
            message: format!(
                "No version exists that matches requirement `{}`",
                version.unquoted()
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }]);
    }

    // Try to find the latest non-prerelease version
    let Some(latest_non_prerelease) = metas.iter().find(|v| {
        Version::parse(&v.version)
            .map(|version| version.pre.is_empty())
            .unwrap_or_default()
    }) else {
        return Ok(Vec::new());
    };
    let latest_name = latest_non_prerelease.name.as_str();
    let Ok(latest_version) = Version::parse(&latest_non_prerelease.version) else {
        return Ok(Vec::new());
    };

    if !version_req.matches(&latest_version) {
        // NOTE: If we have an exact version specified,
        // and it is more recent than the latest non-prerelease, we
        // should not tell the user that a more recent version exists
        if let Ok(exact_version) = Version::parse(version.unquoted()) {
            if exact_version > latest_version {
                return Ok(Vec::new());
            }
        }

        let metadata = CodeActionMetadata::LatestVersion {
            source_uri: doc.uri().clone(),
            source_text: version.quoted().to_string(),
            version_current: version.unquoted().to_string(),
            version_latest: latest_version.to_string(),
        };

        return Ok(vec![Diagnostic {
            source: Some(String::from("Cargo")),
            range: version.range,
            message: format!(
                "A newer version of `{latest_name}` is available.\
                \nThe latest version is `{latest_version}`"
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

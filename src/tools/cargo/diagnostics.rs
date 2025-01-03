use semver::{Version, VersionReq};
use tracing::trace;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::parser::Dependency;

use super::super::shared::*;
use super::{Clients, Document};

pub async fn get_cargo_diagnostics(
    clients: &Clients,
    doc: &Document,
    dep: &Dependency,
) -> Result<Option<Diagnostic>> {
    let Some(version) = dep.spec.contents.version.as_ref() else {
        return Ok(None);
    };
    let Ok(version_req) = VersionReq::parse(version.unquoted()) else {
        return Ok(None);
    };

    trace!("Fetching crate data from crates.io");
    let metadatas = match clients
        .crates
        .get_sparse_index_crate_metadatas(dep.name.unquoted())
        .await
    {
        Ok(v) => v,
        Err(e) => {
            if e.is_not_found_error() {
                return Ok(Some(Diagnostic {
                    source: Some(String::from("Cargo")),
                    range: dep.name.range,
                    message: format!("No package exists with the name `{}`", dep.name.unquoted()),
                    severity: Some(DiagnosticSeverity::ERROR),
                    ..Default::default()
                }));
            } else {
                return Ok(None);
            }
        }
    };

    // Check if the specified package version exists in the index
    if !metadatas.iter().any(|r| {
        Version::parse(&r.version)
            .map(|version| version_req.matches(&version))
            .ok()
            .unwrap_or_default()
    }) {
        return Ok(Some(Diagnostic {
            source: Some(String::from("Cargo")),
            range: version.range,
            message: format!(
                "No version exists that matches requirement `{}`",
                version.unquoted()
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        }));
    }

    // Try to find the latest non-prerelease version
    let Some(latest_non_prerelease) = metadatas.iter().find(|v| {
        Version::parse(&v.version)
            .map(|version| version.pre.is_empty())
            .unwrap_or_default()
    }) else {
        return Ok(None);
    };
    let latest_name = latest_non_prerelease.name.as_str();
    let Ok(latest_version) = Version::parse(&latest_non_prerelease.version) else {
        return Ok(None);
    };

    if !version_req.matches(&latest_version) {
        // NOTE: If we have an exact version specified,
        // and it is more recent than the latest non-prerelease, we
        // should not tell the user that a more recent version exists
        if let Ok(exact_version) = Version::parse(version.unquoted()) {
            if exact_version > latest_version {
                return Ok(None);
            }
        }

        let metadata = CodeActionMetadata::LatestVersion {
            source_uri: doc.uri().clone(),
            source_text: version.quoted().to_string(),
            version_current: version.unquoted().to_string(),
            version_latest: latest_version.to_string(),
        };

        return Ok(Some(Diagnostic {
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
        }));
    }

    Ok(None)
}

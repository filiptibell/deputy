use semver::Version;
use tower_lsp::lsp_types::*;

use crate::clients::*;

use super::super::util::*;
use super::manifest::*;

pub async fn diagnose_dependency(
    clients: &Clients,
    uri: &Url,
    dep: &ManifestDependency,
    range_name: &Range,
    range_version: &Range,
) -> Option<Diagnostic> {
    let spec = match dep.spec() {
        Ok(spec) => spec,
        Err(e) if matches!(e, SpecError::InvalidName(_)) => {
            return Some(Diagnostic {
                source: Some(String::from("Cargo")),
                range: *range_name,
                message: e.to_string(),
                severity: Some(e.diagnostic_severity()),
                ..Default::default()
            })
        }
        Err(e) => {
            return Some(Diagnostic {
                source: Some(String::from("Cargo")),
                range: *range_version,
                message: e.to_string(),
                severity: Some(e.diagnostic_severity()),
                ..Default::default()
            })
        }
    };

    let metadatas = clients.crates.get_index_metadatas(&spec.name).await;
    if metadatas.as_deref().is_err_and(|e| e.is_not_found_error()) {
        return Some(Diagnostic {
            source: Some(String::from("Cargo")),
            range: *range_name,
            message: format!("No package was found for '{}'", spec.name),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }

    let metadatas = metadatas.ok()?;
    if !metadatas.iter().any(|r| {
        Version::parse(&r.version)
            .map(|version| spec.version_req.matches(&version))
            .ok()
            .unwrap_or_default()
    }) {
        return Some(Diagnostic {
            source: Some(String::from("Cargo")),
            range: *range_version,
            message: format!(
                "No package was found matching the version '{}'",
                spec.version_req
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }

    let latest_non_prerelease = metadatas.iter().find(|v| {
        Version::parse(&v.version)
            .map(|version| version.pre.is_empty())
            .unwrap_or_default()
    })?;
    let latest_non_prerelease_name = latest_non_prerelease.name.as_str();
    let latest_non_prerelease_version = Version::parse(&latest_non_prerelease.version).ok()?;
    if !spec.version_req.matches(&latest_non_prerelease_version) {
        // HACK: If we have an exact version specified,
        // and it is more recent than the latest non-prerelease, we
        // should not tell the user that a more recent version exists
        if let Ok(exact_version) = Version::parse(&spec.version) {
            if exact_version > latest_non_prerelease_version {
                return None;
            }
        }
        let metadata = CodeActionMetadata::LatestVersion {
            source_uri: uri.clone(),
            source_text: dep.version_source().to_string(),
            version_current: dep.version_text().to_string(),
            version_latest: latest_non_prerelease_version.to_string(),
        };
        return Some(Diagnostic {
            source: Some(String::from("Cargo")),
            range: *range_version,
            message: format!(
                "A newer version of `{latest_non_prerelease_name}` is available.\
                \nThe latest version is `{latest_non_prerelease_version}`"
            ),
            severity: Some(DiagnosticSeverity::INFORMATION),
            data: Some(
                ResolveContext {
                    uri: uri.clone(),
                    value: metadata,
                }
                .into(),
            ),
            ..Default::default()
        });
    }

    None
}

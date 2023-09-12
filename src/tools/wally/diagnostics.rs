use semver::Version;
use tower_lsp::lsp_types::*;

use crate::clients::*;

use super::super::util::*;
use super::manifest::*;

pub async fn diagnose_dependency(
    clients: &Clients,
    uri: &Url,
    registry_urls: &[String],
    dep: &ManifestDependency,
    range: &Range,
) -> Option<Diagnostic> {
    let spec = match dep.spec() {
        Ok(spec) => spec,
        Err(e) => {
            return Some(Diagnostic {
                source: Some(String::from("Wally")),
                range: *range,
                message: e.to_string(),
                severity: Some(DiagnosticSeverity::ERROR),
                ..Default::default()
            })
        }
    };

    let mut metadatas = None;
    for registry_url in registry_urls {
        let registry_metadatas = clients
            .wally
            .get_index_metadatas(registry_url, &spec.author, &spec.name)
            .await;
        if registry_metadatas.is_ok() {
            metadatas = Some(registry_metadatas);
            break;
        }
    }
    let metadatas = metadatas?;
    if metadatas.as_deref().is_err_and(|e| e.is_not_found_error()) {
        return Some(Diagnostic {
            source: Some(String::from("Wally")),
            range: *range,
            message: format!("No package was found for '{}'", spec.name),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }

    let metadatas = metadatas.ok()?;
    if !metadatas.iter().any(|m| {
        Version::parse(&m.package.version)
            .map(|version| spec.version_req.matches(&version))
            .ok()
            .unwrap_or_default()
    }) {
        return Some(Diagnostic {
            source: Some(String::from("Wally")),
            range: *range,
            message: format!(
                "No package was found matching the version '{}'",
                spec.version_req
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }

    let latest_non_prerelease = metadatas.iter().find(|m| {
        Version::parse(&m.package.version)
            .map(|version| version.pre.is_empty())
            .unwrap_or_default()
    })?;
    let latest_non_prerelease_version =
        Version::parse(&latest_non_prerelease.package.version).ok()?;
    if !spec.version_req.matches(&latest_non_prerelease_version) {
        // HACK: If we have an exact version specified,
        // and it is more recent than the latest non-prerelease, we
        // should not tell the user that a more recent version exists
        if let Some(exact_version) = spec.version {
            if exact_version > latest_non_prerelease_version {
                return None;
            }
        }
        let metadata = CodeActionMetadata::LatestVersion {
            source_uri: uri.clone(),
            source_text: dep.source().to_string(),
            version_current: spec.version_text.to_string(),
            version_latest: latest_non_prerelease_version.to_string(),
        };
        return Some(Diagnostic {
            source: Some(String::from("Wally")),
            range: *range,
            message: format!(
                "A newer version is available.\
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

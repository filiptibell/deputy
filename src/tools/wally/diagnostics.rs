use semver::Version;
use tower_lsp::lsp_types::*;

use crate::clients::*;

use super::super::util::*;
use super::manifest::*;

pub async fn diagnose_dependency(
    clients: &Clients,
    uri: &Url,
    index_url: &str,
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
                severity: Some(e.diagnostic_severity()),
                ..Default::default()
            })
        }
    };

    let authors = clients.wally.get_index_scopes(index_url).await.ok()?;
    if !authors.contains(&spec.author) {
        return Some(Diagnostic {
            source: Some(String::from("Wally")),
            range: *range,
            message: format!("No author exists with the name `{}`", spec.name),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }

    let packages = clients
        .wally
        .get_index_packages(index_url, &spec.author)
        .await
        .ok()?;
    if !packages.contains(&spec.name) {
        return Some(Diagnostic {
            source: Some(String::from("Wally")),
            range: *range,
            message: format!("No package exists with the name `{}`", spec.name),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
    }

    let metadatas = clients
        .wally
        .get_index_metadatas(index_url, &spec.author, &spec.name)
        .await
        .ok()?;
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
        if let Ok(exact_version) = Version::parse(&spec.version) {
            if exact_version > latest_non_prerelease_version {
                return None;
            }
        }
        let name = spec.name.as_str();
        let metadata = CodeActionMetadata::LatestVersion {
            source_uri: uri.clone(),
            source_text: dep.source().to_string(),
            version_current: spec.version.to_string(),
            version_latest: latest_non_prerelease_version.to_string(),
        };
        return Some(Diagnostic {
            source: Some(String::from("Wally")),
            range: *range,
            message: format!(
                "A newer version of `{name}` is available.\
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

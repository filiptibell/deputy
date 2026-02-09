use std::collections::HashMap;

use serde::Deserialize;

use deputy_versioning::PepVersioned;

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadata {
    pub info: RegistryMetadataInfo,
    #[serde(default)]
    pub releases: HashMap<String, Vec<RegistryMetadataRelease>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadataInfo {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub author_email: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub home_page: Option<String>,
    #[serde(default)]
    pub project_url: Option<String>,
    #[serde(default)]
    pub project_urls: Option<HashMap<String, String>>,
    #[serde(default)]
    pub requires_python: Option<String>,
    #[serde(default)]
    pub yanked: bool,
    #[serde(default)]
    pub yanked_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadataRelease {
    #[serde(default)]
    pub yanked: bool,
    #[serde(default)]
    pub yanked_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RegistryMetadataVersion {
    pub version: String,
    pub yanked: bool,
}

impl PepVersioned for RegistryMetadataVersion {
    fn raw_version_string(&self) -> String {
        self.version.clone()
    }

    fn deprecated(&self) -> bool {
        self.yanked
    }
}

impl RegistryMetadata {
    #[allow(clippy::missing_errors_doc)]
    pub fn try_from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Converts the releases map into a flat list of versions.
    ///
    /// A version is considered yanked if all of its release files
    /// are yanked, or if the version has no release files at all.
    #[must_use]
    pub fn versions(&self) -> Vec<RegistryMetadataVersion> {
        self.releases
            .iter()
            .map(|(version, files)| RegistryMetadataVersion {
                version: version.clone(),
                yanked: files.is_empty() || files.iter().all(|f| f.yanked),
            })
            .collect()
    }
}

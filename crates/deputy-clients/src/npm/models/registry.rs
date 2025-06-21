use std::collections::HashMap;

use serde::Deserialize;

use deputy_versioning::Versioned;

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadata {
    #[serde(flatten)]
    pub current_version: RegistryMetadataVersion,
    #[serde(default, rename = "time")]
    pub timestamps: HashMap<String, String>,
    #[serde(default)]
    pub versions: HashMap<String, RegistryMetadataVersion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadataVersion {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<RegistryMetadataLicenseVariant>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<RegistryMetadataRepositoryVariant>,
    #[serde(default)]
    pub author: Option<RegistryMetadataHumanVariant>,
    #[serde(default)]
    pub maintainers: Vec<RegistryMetadataHumanVariant>,
    #[serde(default)]
    pub deprecated: Option<String>,
}

impl Versioned for RegistryMetadataVersion {
    fn raw_version_string(&self) -> String {
        self.version.to_string()
    }

    fn deprecated(&self) -> bool {
        self.deprecated.is_some()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadataLicense {
    #[serde(rename = "type")]
    pub kind: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RegistryMetadataLicenseVariant {
    String(String),
    Full(RegistryMetadataLicense),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadataHuman {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RegistryMetadataHumanVariant {
    String(String),
    Full(RegistryMetadataHuman),
}

impl RegistryMetadataHumanVariant {
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::String(s) => s.as_ref(),
            Self::Full(f) => f.name.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegistryMetadataRepositoryKind {
    Git,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadataRepository {
    #[serde(rename = "type")]
    pub kind: RegistryMetadataRepositoryKind,
    pub url: String,
    #[serde(alias = "directory")]
    pub dir: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RegistryMetadataRepositoryVariant {
    String(String),
    Full(RegistryMetadataRepository),
}

impl RegistryMetadata {
    #[allow(clippy::missing_errors_doc)]
    pub fn try_from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl RegistryMetadataRepositoryVariant {
    #[must_use]
    pub fn url(&self) -> Option<String> {
        match self {
            Self::Full(f) => Some(f.url.clone()),
            Self::String(s) => {
                let (base_url, user, repo, suffix) = match s.trim() {
                    s if s.starts_with("github:") => s
                        .trim_start_matches("github:")
                        .split_once('/')
                        .map(|(u, r)| ("https://github.com/", u, r, "")),
                    s if s.starts_with("gitlab:") => s
                        .trim_start_matches("gitlab:")
                        .split_once('/')
                        .map(|(u, r)| ("https://gitlab.com/", u, r, "")),
                    s if s.starts_with("bitbucket:") => s
                        .trim_start_matches("bitbucket:")
                        .split_once('/')
                        .map(|(u, r)| ("https://bitbucket.org/", u, r, "/overview")),
                    _ => None,
                }?;
                Some(format!("{base_url}{user}/{repo}{suffix}"))
            }
        }
    }
}

use std::collections::HashMap;

use serde::Deserialize;

use crate::util::Versioned;

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadata {
    #[serde(flatten)]
    pub current_version: RegistryMetadataVersion,
    #[serde(rename = "time")]
    pub timestamps: HashMap<String, String>,
    pub versions: HashMap<String, RegistryMetadataVersion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadataVersion {
    pub name: String,
    #[serde(default)]
    pub version: String,
    pub description: Option<String>,
    pub license: Option<RegistryMetadataLicenseVariant>,
    pub homepage: Option<String>,
    pub repository: Option<RegistryMetadataRepositoryVariant>,
    pub author: Option<RegistryMetadataHumanVariant>,
    #[serde(default)]
    pub maintainers: Vec<RegistryMetadataHumanVariant>,
}

impl Versioned for RegistryMetadataVersion {
    fn parse_version(&self) -> Result<semver::Version, semver::Error> {
        self.version.parse()
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
    pub fn _name(&self) -> &str {
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
    pub fn try_from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl RegistryMetadataRepositoryVariant {
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

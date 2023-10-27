use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadata {
    #[serde(flatten)]
    pub current_version: RegistryMetadataVersion,
    #[serde(rename = "time")]
    pub timestamps: HashMap<String, String>,
    pub versions: HashMap<String, RegistryMetadataVersion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadataHuman {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadataDependency {
    pub name: String,
    #[serde(rename = "req")]
    pub version_requirement: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryMetadataVersion {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub author: Option<RegistryMetadataHuman>,
    #[serde(default)]
    pub maintainers: Vec<RegistryMetadataHuman>,
    #[serde(default)]
    pub dependencies: Vec<RegistryMetadataDependency>,
}

impl RegistryMetadata {
    pub fn try_from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

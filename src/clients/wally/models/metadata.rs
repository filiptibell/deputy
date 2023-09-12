use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Metadata {
    pub package: MetadataPackage,
    #[serde(flatten)]
    pub dependencies: MetadataDependencies,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MetadataPackage {
    pub name: String,
    pub version: String,
    pub registry: String,
    pub realm: MetadataRealm,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub private: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MetadataDependencies {
    #[serde(default, rename = "dependencies")]
    pub shared: HashMap<String, String>,
    #[serde(default, rename = "server-dependencies")]
    pub server: HashMap<String, String>,
    #[serde(default, rename = "dev-dependencies")]
    pub dev: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetadataRealm {
    Server,
    Shared,
    Dev,
}

impl Metadata {
    pub fn try_from_lines(lines: Vec<&'_ str>) -> Result<Vec<Self>, serde_json::Error> {
        let mut packages = Vec::new();
        for line in lines {
            match serde_json::from_str(line) {
                Ok(package) => packages.push(package),
                Err(err) => return Err(err),
            }
        }
        Ok(packages)
    }
}

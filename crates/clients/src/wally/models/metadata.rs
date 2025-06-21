use std::collections::HashMap;

use serde::Deserialize;

use shared::Versioned;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Metadata {
    pub package: MetadataPackage,
    #[serde(flatten)]
    pub dependencies: MetadataDependencies,
}

impl Versioned for Metadata {
    fn raw_version_string(&self) -> String {
        self.package.version.to_string()
    }
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
    pub repository: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
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

impl Versioned for MetadataPackage {
    fn raw_version_string(&self) -> String {
        self.version.to_string()
    }
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
    Dev,
    Server,
    Shared,
}

impl Metadata {
    #[allow(clippy::missing_errors_doc)]
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

impl MetadataRealm {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Server => "server",
            Self::Shared => "shared",
        }
    }

    #[must_use]
    pub const fn section_name(self) -> &'static str {
        match self {
            Self::Dev => "dev-dependencies",
            Self::Server => "server-dependencies",
            Self::Shared => "dependencies",
        }
    }

    #[must_use]
    pub const fn get_suggested_realm(self, found_realm: Self) -> Option<Self> {
        use MetadataRealm::{Dev, Server, Shared};

        match (found_realm, self) {
            // Suggest server when placed in shared
            (Server, Shared) => Some(Server),
            // Suggest dev when placed in others
            (Dev, Server | Shared) => Some(Dev),
            // Anything else is fine
            _ => None,
        }
    }
}

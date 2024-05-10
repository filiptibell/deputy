use std::{collections::HashMap, str::FromStr};

use serde::Deserialize;
use tracing::error;

use super::Manifest;

#[derive(Debug, Clone, Deserialize)]
pub struct LockfilePackage {
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Lockfile {
    pub packages: HashMap<String, LockfilePackage>,
}

impl Lockfile {
    pub fn from_manifest_as_fallback(manifest: &Manifest) -> Self {
        let mut packages = HashMap::new();
        for (name, version) in manifest.all_dependencies() {
            if let Ok(version) = version.spec().map(|s| s.version.clone()) {
                packages.insert(name.clone(), LockfilePackage { version });
            }
        }
        Self { packages }
    }
}

// FUTURE: Implement more robust lockfile parsing that supports all lockfile formats
impl FromStr for Lockfile {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = serde_json::from_str::<Lockfile>(s);
        if let Err(e) = &result {
            error!("failed to deserialize npm lockfile - {e}")
        }
        result
    }
}

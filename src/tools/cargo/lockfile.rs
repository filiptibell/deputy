use std::str::FromStr;

use serde::Deserialize;
use tracing::error;

#[derive(Debug, Clone, Deserialize)]
pub struct LockfilePackage {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Lockfile {
    #[serde(rename = "package")]
    pub packages: Vec<LockfilePackage>,
}

impl FromStr for Lockfile {
    type Err = toml::de::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = toml::from_str::<Lockfile>(s);
        if let Err(e) = &result {
            error!("failed to deserialize cargo lockfile - {e}")
        }
        result
    }
}

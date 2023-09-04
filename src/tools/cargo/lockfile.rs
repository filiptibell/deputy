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

impl Lockfile {
    pub fn parse(source: impl AsRef<str>) -> Result<Self, toml::de::Error> {
        let result = toml::from_str::<Lockfile>(source.as_ref());
        if let Err(e) = &result {
            error!("failed to deserialize cargo lockfile - {e}")
        }
        result
    }
}

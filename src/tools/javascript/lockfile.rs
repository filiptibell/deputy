use std::{collections::HashMap, str::FromStr};

use serde::Deserialize;
use tracing::error;

#[derive(Debug, Clone, Deserialize)]
pub struct LockfilePackage {
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Lockfile {
    pub packages: HashMap<String, LockfilePackage>,
}

impl FromStr for Lockfile {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = serde_json::from_str::<Lockfile>(s);
        if let Err(e) = &result {
            error!("failed to deserialize javascript lockfile - {e}")
        }
        result
    }
}

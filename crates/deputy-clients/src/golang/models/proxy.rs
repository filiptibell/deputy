use serde::Deserialize;

use deputy_versioning::Versioned;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ModuleVersion {
    pub version: String,
    pub time: Option<String>,
}

impl Versioned for ModuleVersion {
    fn raw_version_string(&self) -> String {
        self.version.trim_start_matches('v').to_string()
    }
}

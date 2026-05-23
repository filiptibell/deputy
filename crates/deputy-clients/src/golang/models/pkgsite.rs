#![allow(clippy::struct_excessive_bools)]

use serde::{Deserialize, Deserializer};

use deputy_versioning::Versioned;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub path: String,
    pub version: String,
    pub commit_time: Option<String>,
    pub is_latest: bool,
    pub is_redistributable: bool,
    pub is_standard_library: bool,
    pub has_go_mod: bool,
    pub repo_url: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub module_path: String,
    pub version: String,
    pub is_latest: bool,
    pub is_standard_library: bool,
    pub goos: String,
    pub goarch: String,
    pub path: String,
    pub name: String,
    pub synopsis: String,
    pub is_redistributable: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResults {
    #[serde(default, deserialize_with = "deserialize_null_as_default")]
    pub items: Vec<SearchResult>,
    pub total: usize,
    #[serde(default, rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub package_path: String,
    pub module_path: String,
    pub version: String,
    pub synopsis: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Versions {
    #[serde(default, deserialize_with = "deserialize_null_as_default")]
    pub items: Vec<ModuleVersion>,
    pub total: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleVersion {
    pub module_path: String,
    pub version: String,
    pub commit_time: Option<String>,
    pub is_redistributable: bool,
    pub has_go_mod: bool,
    pub latest_version: String,
    pub deprecated: bool,
    pub deprecation_reason: String,
    pub retracted: bool,
    pub retraction_reason: String,
}

impl ModuleVersion {
    #[must_use]
    pub fn is_unusable(&self) -> bool {
        self.deprecated || self.retracted
    }

    #[must_use]
    pub fn unusable_reason(&self) -> Option<&str> {
        if self.deprecated {
            Some(self.deprecation_reason.trim()).filter(|reason| !reason.is_empty())
        } else if self.retracted {
            Some(self.retraction_reason.trim()).filter(|reason| !reason.is_empty())
        } else {
            None
        }
    }
}

impl Versioned for ModuleVersion {
    fn raw_version_string(&self) -> String {
        self.version.trim_start_matches('v').to_string()
    }

    fn deprecated(&self) -> bool {
        self.is_unusable()
    }
}

fn deserialize_null_as_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

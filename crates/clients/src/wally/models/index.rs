use serde::Deserialize;

/**
    Configuration for a Wally index.

    Located at the root of the index repository, as `config.json`.
*/
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct IndexConfig {
    #[serde(rename = "api")]
    pub api_url: String,
    #[serde(default)]
    pub fallback_registries: Vec<String>,
}

/**
    Configuration for a Wally index owners file.

    Located at the root of each scope in the index repository, as `owners.json`.
*/
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(transparent)]
pub struct IndexOwners {
    github_user_ids: Vec<u64>,
}

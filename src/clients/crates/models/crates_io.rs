use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CrateDataSingle {
    #[serde(rename = "crate")]
    pub inner: CrateData,
    #[serde(default)]
    pub versions: Vec<CrateDataVersion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateDataMulti {
    #[serde(rename = "crates")]
    pub inner: Vec<CrateData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateData {
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(flatten)]
    pub links: CrateDataLinks,
    #[serde(flatten)]
    pub downloads: CrateDataDownloads,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateDataLinks {
    pub documentation: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateDataDownloads {
    #[serde(rename = "downloads")]
    pub total_count: u64,
    #[serde(rename = "recent_downloads")]
    pub recent_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateDataVersion {
    pub id: u64,
    #[serde(alias = "crate")]
    pub name: String,
    #[serde(alias = "num")]
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub downloads: u64,
    pub features: HashMap<String, Vec<String>>,
}

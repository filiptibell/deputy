use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CrateDataSingle {
    #[serde(rename = "crate")]
    pub inner: CrateDataInner,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateDataMulti {
    #[serde(rename = "crates")]
    pub inner: Vec<CrateDataInner>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrateDataInner {
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

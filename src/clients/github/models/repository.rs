use serde::Deserialize;

use crate::util::Versioned;

#[derive(Debug, Clone, Deserialize)]
pub struct RepositoryMetrics {
    pub description: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepositoryRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: Option<String>,
    pub published_at: Option<String>,
    pub assets: Vec<RepositoryReleaseAsset>,
}

impl Versioned for RepositoryRelease {
    fn parse_version(&self) -> Result<semver::Version, semver::Error> {
        self.tag_name.trim_start_matches('v').parse()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepositoryReleaseAsset {
    pub name: String,
    pub label: Option<String>,
    pub content_type: String,
    pub size: i64,
    pub download_count: i64,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

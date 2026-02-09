use deputy_versioning::Version;

#[derive(Debug, Clone)]
pub struct LocalMetadata {
    pub version: Option<Version>,
    pub features: Vec<String>,
}

use serde::Deserialize;

use deputy_versioning::PepVersioned;

#[derive(Debug, Clone, Deserialize)]
pub struct SimpleMetadata {
    pub name: String,
    #[serde(default)]
    pub versions: Vec<String>,
    #[serde(default)]
    pub files: Vec<SimpleMetadataFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SimpleMetadataFile {
    pub filename: String,
    #[serde(default)]
    pub yanked: bool,
}

#[derive(Debug, Clone)]
pub struct SimpleMetadataVersion {
    pub version: String,
    pub yanked: bool,
}

impl PepVersioned for SimpleMetadataVersion {
    fn raw_version_string(&self) -> String {
        self.version.clone()
    }

    fn deprecated(&self) -> bool {
        self.yanked
    }
}

impl SimpleMetadata {
    #[allow(clippy::missing_errors_doc)]
    pub fn try_from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Converts the metadata into a flat list of versions.
    ///
    /// A version is considered yanked if all of its files
    /// are yanked. Files are matched to versions by checking
    /// if the filename starts with `{name}-{version}`.
    #[must_use]
    pub fn versions(&self) -> Vec<SimpleMetadataVersion> {
        let name_lower = self.name.to_lowercase();
        self.versions
            .iter()
            .map(|version| {
                let prefix = format!("{name_lower}-{version}");
                let matching_files: Vec<_> = self
                    .files
                    .iter()
                    .filter(|f| f.filename.to_lowercase().starts_with(&prefix))
                    .collect();
                SimpleMetadataVersion {
                    version: version.clone(),
                    yanked: !matching_files.is_empty() && matching_files.iter().all(|f| f.yanked),
                }
            })
            .collect()
    }
}

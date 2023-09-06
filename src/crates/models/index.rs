use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct IndexMetadata {
    pub name: String,
    #[serde(rename = "vers")]
    pub version: String,
    #[serde(rename = "deps")]
    pub dependencies: Vec<IndexMetadataDependency>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexMetadataDependency {
    pub name: String,
    #[serde(rename = "req")]
    pub version_requirement: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
}

impl IndexMetadata {
    pub fn try_from_lines(lines: Vec<&'_ str>) -> Result<Vec<Self>, serde_json::Error> {
        let mut packages = Vec::new();
        for line in lines {
            match serde_json::from_str(line) {
                Ok(package) => packages.push(package),
                Err(err) => return Err(err),
            }
        }
        Ok(packages)
    }
}

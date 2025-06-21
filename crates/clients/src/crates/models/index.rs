use std::collections::HashMap;

use serde::Deserialize;

use versioning::Versioned;

#[derive(Debug, Clone, Deserialize)]
pub struct IndexMetadata {
    pub name: String,
    #[serde(alias = "vers")]
    pub version: String,
    #[serde(default, alias = "deps")]
    pub dependencies: Vec<IndexMetadataDependency>,
    #[serde(default, alias = "feats")]
    pub features: HashMap<String, Vec<String>>,
    #[serde(default, alias = "feats2")]
    pub features2: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub yanked: bool,
}

impl Versioned for IndexMetadata {
    fn raw_version_string(&self) -> String {
        self.version.to_string()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexMetadataDependency {
    pub name: String,
    #[serde(alias = "req")]
    pub version_requirement: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
}

impl IndexMetadata {
    #[allow(clippy::missing_errors_doc)]
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

    /**
        Returns a sorted list of all features for the package,
        including ones implicitly added by optional dependencies.
    */
    #[must_use]
    pub fn all_features(&self) -> Vec<&str> {
        let mut features = Vec::new();

        for dep in &self.dependencies {
            if dep.optional
                && !explicitly_mentions_feature_dep(&self.features, &dep.name)
                && !explicitly_mentions_feature_dep(&self.features2, &dep.name)
            {
                features.push(dep.name.as_str());
            }
        }

        for feature_name in self.features.keys() {
            features.push(feature_name);
        }
        for feature_name in self.features2.keys() {
            features.push(feature_name);
        }

        features.sort_unstable();
        features.dedup();

        features
    }
}

fn explicitly_mentions_feature_dep(features: &HashMap<String, Vec<String>>, feature: &str) -> bool {
    features.values().any(|enables| {
        enables.iter().any(|enabled_feature_spec| {
            enabled_feature_spec
                .strip_prefix("dep:")
                .is_some_and(|enabled_feature_name| enabled_feature_name == feature)
        })
    })
}

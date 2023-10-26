use std::{collections::HashMap, ops::Range, str::FromStr};

use tracing::error;

use super::util::*;
use crate::lang::json::*;
use crate::lang::LangString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestDependencyKind {
    Default,
    Dev,
    Build,
    Optional,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManifestDependency {
    kind: ManifestDependencyKind,
    spec: Spec<JsonString, JsonString, JsonValue>,
}

impl ManifestDependency {
    fn from_json_values(key: &JsonString, value: &JsonValue) -> Option<Self> {
        let version = value.as_string()?;
        Some(Self {
            kind: ManifestDependencyKind::Default,
            spec: Spec::from_key_value_pair(key, version),
        })
    }

    pub fn _spec(&self) -> Result<SpecJavaScript, SpecError> {
        self.spec.as_javascript()
    }

    pub fn _name_span(&self) -> Range<usize> {
        self.spec.key_span()
    }

    pub fn _name_text(&self) -> &str {
        self.spec.key_text()
    }

    pub fn _version_span(&self) -> Range<usize> {
        self.spec.value_span()
    }

    pub fn _version_source(&self) -> &str {
        self.spec.value_source()
    }

    pub fn _version_text(&self) -> &str {
        self.spec.value_text()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Manifest {
    pub dependencies: HashMap<String, ManifestDependency>,
    pub dev_dependencies: HashMap<String, ManifestDependency>,
    pub build_dependencies: HashMap<String, ManifestDependency>,
    pub optional_dependencies: HashMap<String, ManifestDependency>,
}

impl Manifest {
    fn from_json_value(value: &JsonValue) -> Option<Self> {
        let root = match value.as_map() {
            None => return None,
            Some(t) => t,
        };

        let mut manifest = Manifest::default();

        let fill = |map_key: &str,
                    map: &mut HashMap<String, ManifestDependency>,
                    kind: ManifestDependencyKind| {
            if let Some((_, deps)) = root.find(map_key) {
                if let Some(deps_table) = deps.as_map() {
                    for (k, v) in deps_table.as_ref().iter() {
                        if let Some(mut tool) = ManifestDependency::from_json_values(k, v) {
                            tool.kind = kind;
                            map.insert(k.value().to_string(), tool);
                        }
                    }
                }
            }
        };

        fill(
            "dependencies",
            &mut manifest.dependencies,
            ManifestDependencyKind::Default,
        );
        fill(
            "devDependencies",
            &mut manifest.dev_dependencies,
            ManifestDependencyKind::Dev,
        );
        fill(
            "buildDependencies",
            &mut manifest.build_dependencies,
            ManifestDependencyKind::Build,
        );
        fill(
            "optionalDependencies",
            &mut manifest.optional_dependencies,
            ManifestDependencyKind::Optional,
        );

        Some(manifest)
    }

    pub fn parse(source: impl AsRef<str>) -> Result<Self, JsonError> {
        match JsonValue::new(source.as_ref()) {
            Ok(value) => Ok(Self::from_json_value(&value).expect("Json root should be a map")),
            Err(e) => {
                error!("failed to deserialize tools manifest - {e}");
                Err(e)
            }
        }
    }
}

impl FromStr for Manifest {
    type Err = JsonError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

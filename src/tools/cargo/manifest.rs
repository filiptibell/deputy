use std::{collections::HashMap, ops::Range, str::FromStr};

use tracing::error;

use super::util::*;
use crate::lang::toml::*;
use crate::lang::LangString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestDependencyKind {
    Default,
    Dev,
    Build,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManifestDependency {
    kind: ManifestDependencyKind,
    spec: Spec<TomlString, TomlString, TomlValue>,
}

impl ManifestDependency {
    fn from_toml_values(key: &TomlString, value: &TomlValue) -> Option<Self> {
        let version = value.as_string().or_else(|| {
            let (_, version) = value.as_table().and_then(|t| t.find("version"))?;
            if version.kind().is_string() {
                Some(version.as_string().unwrap())
            } else {
                None
            }
        })?;

        let features = value
            .as_table()
            .and_then(|t| t.find("features"))
            .map(|f| f.1)
            .and_then(|t| t.as_table());

        if let Some(features) = features {
            Some(Self {
                kind: ManifestDependencyKind::Default,
                spec: Spec::from_key_value_pair_with_extras(
                    key,
                    version,
                    &TomlValue::Table(Box::new(features.clone())),
                ),
            })
        } else {
            Some(Self {
                kind: ManifestDependencyKind::Default,
                spec: Spec::from_key_value_pair(key, version),
            })
        }
    }

    pub fn spec(&self) -> Result<SpecCargo, SpecError> {
        self.spec.as_cargo()
    }

    pub fn name_span(&self) -> Range<usize> {
        self.spec.key_span()
    }

    pub fn name_text(&self) -> &str {
        self.spec.key_text()
    }

    pub fn version_span(&self) -> Range<usize> {
        self.spec.value_span()
    }

    pub fn version_source(&self) -> &str {
        self.spec.value_source()
    }

    pub fn version_text(&self) -> &str {
        self.spec.value_text()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Manifest {
    pub dependencies: HashMap<String, ManifestDependency>,
    pub dev_dependencies: HashMap<String, ManifestDependency>,
    pub build_dependencies: HashMap<String, ManifestDependency>,
}

impl Manifest {
    fn from_toml_value(value: &TomlValue) -> Option<Self> {
        let root = match value.as_table() {
            None => return None,
            Some(t) => t,
        };

        let mut manifest = Manifest::default();

        let fill = |map_key: &str,
                    map: &mut HashMap<String, ManifestDependency>,
                    kind: ManifestDependencyKind| {
            if let Some((_, deps)) = root.find(map_key) {
                if let Some(deps_table) = deps.as_table() {
                    for (k, v) in deps_table.as_ref().iter() {
                        if let Some(mut tool) = ManifestDependency::from_toml_values(k, v) {
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
            "dev-dependencies",
            &mut manifest.dev_dependencies,
            ManifestDependencyKind::Dev,
        );
        fill(
            "dev_dependencies",
            &mut manifest.dev_dependencies,
            ManifestDependencyKind::Dev,
        );
        fill(
            "build-dependencies",
            &mut manifest.build_dependencies,
            ManifestDependencyKind::Build,
        );
        fill(
            "build_dependencies",
            &mut manifest.build_dependencies,
            ManifestDependencyKind::Build,
        );

        Some(manifest)
    }

    pub fn parse(source: impl AsRef<str>) -> Result<Self, TomlError> {
        match TomlValue::new(source.as_ref()) {
            Ok(value) => Ok(Self::from_toml_value(&value).expect("Toml root should be a table")),
            Err(e) => {
                error!("failed to deserialize cargo manifest - {e}");
                Err(e)
            }
        }
    }
}

impl FromStr for Manifest {
    type Err = TomlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

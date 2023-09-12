use std::{collections::HashMap, ops::Range, str::FromStr};

use tracing::error;

use super::util::*;
use crate::util::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestDependencyKind {
    Default,
    Dev,
    Build,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestDependency {
    kind: ManifestDependencyKind,
    spec: Spec,
}

impl ManifestDependency {
    fn from_toml_values(key: &TomlString, value: &TomlValue) -> Option<Self> {
        let version = value.as_string().or_else(|| {
            match value.as_table().and_then(|t| t.find("version")) {
                Some((_, version)) if version.kind().is_string() => {
                    Some(version.as_string().unwrap())
                }
                _ => None,
            }
        })?;

        Some(Self {
            kind: ManifestDependencyKind::Default,
            spec: Spec::from_key_value_pair(key, version),
        })
    }

    pub fn spec(&self) -> Result<SpecCargo, SpecError> {
        self.spec.as_cargo()
    }

    pub fn name_span(&self) -> Range<usize> {
        self.spec.key_span()
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
        let tab = match value.as_table() {
            None => return None,
            Some(t) => t,
        };

        let mut manifest = Manifest::default();

        let fill = |map_key: &str,
                    map: &mut HashMap<String, ManifestDependency>,
                    kind: ManifestDependencyKind| {
            if let Some((_, deps)) = tab.find(map_key) {
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

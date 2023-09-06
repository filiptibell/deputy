use std::{collections::HashMap, ops::Range, str::FromStr};

use tracing::error;

use super::dependency_spec::*;
use super::util::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestDependency {
    name: TomlString,
    version: TomlString,
}

impl ManifestDependency {
    fn from_toml_values(key: TomlString, value: &TomlValue) -> Option<Self> {
        let version = value.as_string().cloned().or_else(|| {
            match value.as_table().and_then(|t| t.find("version")) {
                Some((_, version)) if version.kind().is_string() => {
                    Some(version.as_string().unwrap().clone())
                }
                _ => None,
            }
        })?;
        Some(Self { name: key, version })
    }

    pub fn spec(&self) -> Result<DependencySpec, DependencySpecError> {
        DependencySpec::parse(self.name.value(), self.version.value())
    }

    pub fn name_span(&self) -> Range<usize> {
        self.name.span()
    }

    pub fn version_span(&self) -> Range<usize> {
        self.version.span()
    }

    pub fn version_source(&self) -> &str {
        self.version.source()
    }

    pub fn version_text(&self) -> &str {
        self.version.value()
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
        match value.as_table() {
            None => None,
            Some(t) => {
                let mut manifest = Manifest::default();

                let fill = |map_key: &str, map: &mut HashMap<String, ManifestDependency>| {
                    if let Some((_, deps)) = t.find(map_key) {
                        if let Some(deps_table) = deps.as_table() {
                            for (k, v) in deps_table.as_ref().iter() {
                                if let Some(tool) =
                                    ManifestDependency::from_toml_values(k.clone(), v)
                                {
                                    map.insert(k.value().to_string(), tool);
                                }
                            }
                        }
                    }
                };

                fill("dependencies", &mut manifest.dependencies);
                fill("dev-dependencies", &mut manifest.dev_dependencies);
                fill("dev_dependencies", &mut manifest.dev_dependencies);
                fill("build-dependencies", &mut manifest.build_dependencies);
                fill("build_dependencies", &mut manifest.build_dependencies);

                Some(manifest)
            }
        }
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

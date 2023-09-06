use std::{collections::HashMap, ops::Range, str::FromStr};

use tracing::error;

use super::util::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestDependency {
    Plain(TomlString),
    Struct { version: TomlString },
}

impl ManifestDependency {
    fn from_toml_value(value: &TomlValue) -> Option<Self> {
        value
            .as_string()
            .map(|s| Self::Plain(s.clone()))
            .or_else(|| match value.as_table().and_then(|t| t.find("version")) {
                Some((_, version)) if version.kind().is_string() => Some(Self::Struct {
                    version: version.as_string().unwrap().clone(),
                }),
                _ => None,
            })
    }

    pub fn version_span(&self) -> Range<usize> {
        match self {
            Self::Plain(s) => s.span(),
            Self::Struct { version } => version.span(),
        }
    }

    pub fn version_source(&self) -> &str {
        match self {
            Self::Plain(s) => s.source(),
            Self::Struct { version } => version.source(),
        }
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
                if let Some((_, deps)) = t.find("dependencies") {
                    if let Some(deps_table) = deps.as_table() {
                        for (k, v) in deps_table.as_ref().iter() {
                            if let Some(tool) = ManifestDependency::from_toml_value(v) {
                                manifest.dependencies.insert(k.value().to_string(), tool);
                            }
                        }
                    }
                }
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

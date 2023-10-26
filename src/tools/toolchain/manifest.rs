use std::collections::HashMap;
use std::ops::Range;
use std::str::FromStr;

use tracing::error;

use super::util::*;
use crate::lang::toml::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ManifestTool {
    spec: Spec,
}

impl ManifestTool {
    fn from_toml_values(key: &TomlString, value: &TomlValue) -> Option<Self> {
        value.as_string().map(|s| Self {
            spec: Spec::from_key_value_pair(key, s),
        })
    }

    pub fn spec(&self) -> Result<SpecAftman, SpecError> {
        self.spec.as_aftman()
    }

    pub fn span(&self) -> Range<usize> {
        self.spec.value_span()
    }

    pub fn source(&self) -> &str {
        self.spec.value_source()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Manifest {
    pub tools: HashMap<String, ManifestTool>,
}

impl Manifest {
    fn from_toml_value(value: &TomlValue) -> Option<Self> {
        let tab = match value.as_table() {
            None => return None,
            Some(t) => t,
        };

        let mut manifest = Manifest::default();
        if let Some((_, tools)) = tab.find("tools") {
            if let Some(tools_table) = tools.as_table() {
                for (k, v) in tools_table.as_ref().iter() {
                    if let Some(tool) = ManifestTool::from_toml_values(k, v) {
                        manifest.tools.insert(k.value().to_string(), tool);
                    }
                }
            }
        }
        Some(manifest)
    }

    pub fn parse(source: impl AsRef<str>) -> Result<Self, TomlError> {
        match TomlValue::new(source.as_ref()) {
            Ok(value) => Ok(Self::from_toml_value(&value).expect("Toml root should be a table")),
            Err(e) => {
                error!("failed to deserialize tools manifest - {e}");
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

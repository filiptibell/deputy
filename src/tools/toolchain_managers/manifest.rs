use std::{collections::HashMap, fmt, ops::Range, str::FromStr};

use serde::Deserialize;
use serde_spanned::Spanned;
use tracing::error;

use super::tool_spec::*;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct ManifestTool(Spanned<String>);

impl ManifestTool {
    pub fn spec(&self) -> Result<ToolSpec, ToolSpecError> {
        ToolSpec::from_str(self.0.as_ref())
    }

    pub fn span(&self) -> Range<usize> {
        self.0.span()
    }
}

impl fmt::Display for ManifestTool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub tools: HashMap<String, ManifestTool>,
}

impl Manifest {
    pub fn parse_aftman(source: impl AsRef<str>) -> Result<Self, toml::de::Error> {
        let result = toml::from_str::<Manifest>(source.as_ref());
        if let Err(e) = &result {
            error!("failed to deserialize tool manifest - {e}")
        }
        result
    }
}

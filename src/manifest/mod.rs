use std::ops::Range;

use crate::toml::*;

#[derive(Debug, Clone)]
pub struct ManifestToolsHeader {
    pub span: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct ManifestToolsMap {
    pub tools: Vec<ManifestTool>,
}

#[derive(Debug, Clone)]
pub struct ManifestTool {
    pub key_span: Range<usize>,
    pub key_text: String,
    pub val_span: Range<usize>,
    pub val_text: String,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub source: String,
    pub tools_header: ManifestToolsHeader,
    pub tools_map: ManifestToolsMap,
}

impl Manifest {
    pub fn parse(source: impl Into<String>) -> ParserResult<Self> {
        let _source = source.into();

        Err(ParserError::external("unimplemented"))
    }
}

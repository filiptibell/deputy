use lsp_types::Url;

use crate::manifest::*;

pub struct Document {
    pub uri: Url,
    pub version: Option<i32>,
    pub manifest: Manifest,
}

impl From<(Url, Manifest)> for Document {
    fn from((uri, manifest): (Url, Manifest)) -> Self {
        Self {
            uri,
            version: None,
            manifest,
        }
    }
}

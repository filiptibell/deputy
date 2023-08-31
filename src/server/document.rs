#![allow(dead_code)]

use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex as AsyncMutex;

use tower_lsp::lsp_types::*;

use crate::util::*;

pub type Documents = Arc<AsyncMutex<HashMap<Url, Document>>>;

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Url,
    pub name: String,
    pub version: i32,
    pub text: String,
}

#[derive(Debug, Default, Clone)]
pub struct DocumentBuilder {
    uri: Option<Url>,
    name: Option<String>,
    version: Option<i32>,
    text: Option<String>,
}

impl DocumentBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_uri(self, uri: impl Into<Url>) -> Self {
        Self {
            uri: Some(uri.into()),
            ..self
        }
    }

    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    pub fn with_version(self, version: impl Into<i32>) -> Self {
        Self {
            version: Some(version.into()),
            ..self
        }
    }

    pub fn with_text(self, text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..self
        }
    }

    pub fn build(self) -> Document {
        let uri = self.uri.expect("Missing uri");
        let name = self.name.unwrap_or_else(|| match uri_to_file_name(&uri) {
            None => panic!("Encountered document without file name"),
            Some(f) => f,
        });
        Document {
            uri,
            name,
            version: self.version.unwrap_or(i32::MIN),
            text: self.text.unwrap_or_default(),
        }
    }
}

#![allow(dead_code)]

use std::{collections::HashMap, sync::Arc};

use lsp_document::{IndexedText, TextAdapter, TextMap};
use tokio::sync::Mutex as AsyncMutex;

use tower_lsp::lsp_types::*;

use crate::util::*;

type StdRange = std::ops::Range<usize>;

pub type Documents = Arc<AsyncMutex<HashMap<Url, Document>>>;

#[derive(Debug, Clone)]
pub struct Document {
    uri: Url,
    name: String,
    version: i32,
    text: IndexedText<String>,
}

impl Document {
    pub fn uri(&self) -> &Url {
        &self.uri
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn as_str(&self) -> &str {
        &self.text.text
    }

    pub fn lsp_position_to_offset(&self, position: Position) -> usize {
        let pos = self.text.lsp_pos_to_pos(&position).unwrap();
        self.text.pos_to_offset(&pos).unwrap()
    }

    pub fn lsp_position_from_offset(&self, offset: usize) -> Position {
        let pos = self.text.offset_to_pos(offset).unwrap();
        self.text.pos_to_lsp_pos(&pos).unwrap()
    }

    pub fn lsp_range_from_range(&self, range: StdRange) -> Range {
        let start = self.lsp_position_from_offset(range.start);
        let end = self.lsp_position_from_offset(range.end);
        Range::new(start, end)
    }

    pub fn lsp_range_to_range(&self, range: Range) -> StdRange {
        let start = self.lsp_position_to_offset(range.start);
        let end = self.lsp_position_to_offset(range.end);
        StdRange { start, end }
    }

    pub fn create_edit(&self, range: StdRange, new_text: impl Into<String>) -> TextEdit {
        TextEdit {
            range: self.lsp_range_from_range(range),
            new_text: new_text.into(),
        }
    }

    pub fn set_version(&mut self, version: impl Into<i32>) {
        self.version = version.into();
    }

    pub fn apply_change(&mut self, change: TextDocumentContentChangeEvent) {
        let change = self.text.lsp_change_to_change(change).unwrap();
        let replaced = lsp_document::apply_change(&self.text, change);
        self.text = IndexedText::new(replaced);
    }
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
        let name = self.name.unwrap_or_else(|| match uri.file_name() {
            None => panic!("Encountered document without file name"),
            Some(f) => f,
        });
        Document {
            uri,
            name,
            version: self.version.unwrap_or(i32::MIN),
            text: IndexedText::new(self.text.unwrap_or_default()),
        }
    }
}

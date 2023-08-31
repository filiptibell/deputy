#![allow(dead_code)]

use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex as AsyncMutex;

use tower_lsp::lsp_types::*;

use crate::util::*;

type StdRange = std::ops::Range<usize>;

pub type Documents = Arc<AsyncMutex<HashMap<Url, Document>>>;

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Url,
    pub name: String,
    pub version: i32,
    pub text: String,
}

impl Document {
    pub fn position_to_offset(&self, position: Position) -> usize {
        if position.line == 0 {
            return 1 + position.character as usize;
        }

        let last_newline_offset = self
            .text
            .char_indices()
            .filter(|&(_, c)| c == '\n')
            .nth((position.line - 1) as usize)
            .map(|(index, _)| index)
            .expect("Invalid position");

        let mut offset = 0;
        offset += last_newline_offset;
        offset += 1;
        offset += position.character as usize;
        offset
    }

    pub fn offset_to_position(&self, offset: usize) -> Position {
        let mut newline_count = 0;
        let mut newline_last_idx = 0;
        for (index, char) in self.text.char_indices() {
            if index >= offset {
                break;
            }
            if char == '\n' {
                newline_count += 1;
                newline_last_idx = index;
            }
        }

        Position::new(newline_count, (offset - newline_last_idx - 1) as u32)
    }

    pub fn offset_range_to_range(&self, range: StdRange) -> Range {
        let start = self.offset_to_position(range.start);
        let end = self.offset_to_position(range.end);
        Range::new(start, end)
    }

    pub fn range_to_offset_range(&self, range: Range) -> StdRange {
        let start = self.position_to_offset(range.start);
        let end = self.position_to_offset(range.end);
        StdRange { start, end }
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

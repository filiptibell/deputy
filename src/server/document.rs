#![allow(dead_code)]

use std::sync::Arc;

use dashmap::DashMap;
use lsp_document::{IndexedText, TextAdapter, TextMap};

use tower_lsp::lsp_types::*;

use crate::{parser::TreeSitterDocument, util::*};

type Span = std::ops::Range<usize>;

pub type Documents = Arc<DashMap<Url, Document>>;

#[derive(Debug, Clone)]
pub struct Document {
    uri: Url,
    name: String,
    version: i32,
    opened: bool,
    text: IndexedText<String>,
    inner: TreeSitterDocument,
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

    pub fn inner(&self) -> &TreeSitterDocument {
        &self.inner
    }

    pub fn lsp_position_to_offset(&self, position: Position) -> usize {
        let pos = self.text.lsp_pos_to_pos(&position).unwrap();
        self.text.pos_to_offset(&pos).unwrap()
    }

    pub fn lsp_position_from_offset(&self, offset: usize) -> Position {
        let pos = self.text.offset_to_pos(offset).unwrap();
        self.text.pos_to_lsp_pos(&pos).unwrap()
    }

    pub fn lsp_range_from_span(&self, span: Span) -> Range {
        let start = self.lsp_position_from_offset(span.start);
        let end = self.lsp_position_from_offset(span.end);
        Range::new(start, end)
    }

    pub fn lsp_range_to_span(&self, range: Range) -> Span {
        let start = self.lsp_position_to_offset(range.start);
        let end = self.lsp_position_to_offset(range.end);
        Span { start, end }
    }

    pub fn create_edit(&self, range: Range, new_text: impl Into<String>) -> TextEdit {
        let new_text = new_text.into();
        tracing::trace!(
            "Created edit: '{}' at range '{}:{} -> {}:{}' becomes '{}'",
            &self.as_str()[self.lsp_range_to_span(range)],
            range.start.line,
            range.start.character,
            range.end.line,
            range.end.character,
            new_text
        );
        TextEdit { range, new_text }
    }

    pub fn create_substring_edit(
        &self,
        line: u32,
        substring: impl AsRef<str>,
        replacement: impl Into<String>,
    ) -> TextEdit {
        let substring = substring.as_ref();
        let replacement = replacement.into();

        let range = self.text.line_range(line).expect("invalid line");
        let slice = &self.as_str()[self.lsp_range_to_span(Range {
            start: Position {
                line: range.start.line,
                character: 0,
            },
            end: Position {
                line: range.end.line,
                character: range.end.col,
            },
        })];

        let Some(offset) = slice.find(substring) else {
            panic!(
                "Invalid substring edit!\
                \nLine: {slice}\
                \nSubstring: {substring}\
                \nReplacement: {replacement}"
            )
        };
        let edit_range = Range {
            start: Position {
                line: range.start.line,
                character: offset as u32,
            },
            end: Position {
                line: range.end.line,
                character: (offset + substring.len()) as u32,
            },
        };

        self.create_edit(edit_range, replacement)
    }

    pub fn set_version(&mut self, version: impl Into<i32>) {
        self.version = version.into();
    }

    pub fn set_opened(&mut self, opened: bool) {
        self.opened = opened;
    }

    pub fn set_text(&mut self, new_text: impl Into<String>) {
        let text = new_text.into();
        self.text = IndexedText::new(text.clone());
        self.inner.set_contents(text);
    }

    pub fn apply_change(&mut self, change: TextDocumentContentChangeEvent) {
        let change = self.text.lsp_change_to_change(change).unwrap();
        let replaced = lsp_document::apply_change(&self.text, change);
        self.text = IndexedText::new(replaced.clone());
        self.inner.set_contents(replaced);
    }
}

#[derive(Debug, Default, Clone)]
pub struct DocumentBuilder {
    uri: Option<Url>,
    name: Option<String>,
    version: Option<i32>,
    opened: Option<bool>,
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

    pub fn with_opened(self) -> Self {
        Self {
            opened: Some(true),
            ..self
        }
    }

    pub fn build(self) -> Document {
        let uri = self.uri.expect("Missing uri");
        let name = self.name.unwrap_or_else(|| match uri.file_name() {
            None => panic!("Encountered document without file name"),
            Some(f) => f,
        });

        let text = IndexedText::new(self.text.clone().unwrap_or_default());
        let inner = TreeSitterDocument::new(uri.clone(), self.text.unwrap_or_default())
            .expect("encountered unexpected file name with no corresponding language");

        Document {
            uri,
            name,
            version: self.version.unwrap_or(i32::MIN),
            opened: self.opened.unwrap_or(false),
            text,
            inner,
        }
    }
}

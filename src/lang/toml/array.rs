use std::ops::Range;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TomlArray {
    pub(super) span: Range<usize>,
    pub(super) source: String,
    pub(super) entries: Vec<TomlValue>,
}

impl TomlArray {
    pub(super) fn from_node(node: Node, source: impl AsRef<str>) -> Option<Self> {
        match node.as_array() {
            None => None,
            Some(array) => {
                // NOTE: Node is guaranteed to have at least one text range
                let mut text_range = node.text_ranges().next().unwrap();
                for range in node.text_ranges() {
                    text_range = text_range.cover(range);
                }

                let span = Range {
                    start: u32::from(text_range.start()) as usize,
                    end: u32::from(text_range.end()) as usize,
                };

                let source = source.as_ref();
                let text = source[span.clone()].to_string();

                let entries = array
                    .items()
                    .read()
                    .iter()
                    .map(|node| TomlValue::from_node(node.clone(), source))
                    .collect();

                Some(Self {
                    span,
                    source: text,
                    entries,
                })
            }
        }
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    pub fn entries(&self) -> &[TomlValue] {
        self.entries.as_ref()
    }
}

impl AsRef<[TomlValue]> for TomlArray {
    fn as_ref(&self) -> &[TomlValue] {
        &self.entries
    }
}

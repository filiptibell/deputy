use std::ops::{Deref, Range};

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TomlArray {
    pub(super) span: Range<usize>,
    pub(super) entries: Vec<TomlValue>,
}

impl TomlArray {
    pub(super) fn from_node(node: Node) -> Option<Self> {
        match node.as_array() {
            None => None,
            Some(table) => {
                let mut range_first = None;
                let mut range_last = None;
                for range in node.text_ranges() {
                    if range_first.is_none() {
                        range_first = Some(range)
                    } else {
                        range_last = Some(range)
                    }
                }

                let span = Range {
                    start: u32::from(range_first.unwrap().start()) as usize,
                    end: u32::from(range_last.unwrap().end()) as usize,
                };

                let entries = table
                    .items()
                    .read()
                    .iter()
                    .map(|node| TomlValue::from_node(node.clone()))
                    .collect();

                Some(Self { span, entries })
            }
        }
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

impl Deref for TomlArray {
    type Target = Vec<TomlValue>;
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

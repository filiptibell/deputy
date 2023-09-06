use std::ops::{Deref, Range};

use taplo::dom::node::Key;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TomlString {
    pub(super) span: Range<usize>,
    pub(super) value: String,
}

impl TomlString {
    pub(super) fn from_key(key: Key) -> Self {
        let range = key.text_ranges().next().unwrap();
        let span = range_to_span(range);
        let value = key.value().to_string();
        Self { span, value }
    }

    pub(super) fn from_node(node: Node) -> Option<Self> {
        match node.as_str() {
            None => None,
            Some(string) => {
                let range = node.text_ranges().next().unwrap();
                let span = range_to_span(range);
                let value = string.value().to_string();
                Some(Self { span, value })
            }
        }
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

impl Deref for TomlString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

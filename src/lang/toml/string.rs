use std::ops::Range;

use taplo::dom::node::Key;

use crate::lang::{LangString, LangValue};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TomlString {
    pub(super) span: Range<usize>,
    pub(super) source: String,
    pub(super) value: String,
}

impl TomlString {
    pub(super) fn from_key(key: Key, source: impl AsRef<str>) -> Self {
        let range = key.text_ranges().next().unwrap();
        let span = range_to_span(range);

        let source = source.as_ref();
        let text = source[span.clone()].to_string();

        let value = key.value().to_string();
        Self {
            span,
            source: text,
            value,
        }
    }

    pub(super) fn from_node(node: Node, source: impl AsRef<str>) -> Option<Self> {
        match node.as_str() {
            None => None,
            Some(string) => {
                let range = node.text_ranges().next().unwrap();
                let span = range_to_span(range);

                let source = source.as_ref();
                let text = source[span.clone()].to_string();

                let value = string.value().to_string();
                Some(Self {
                    span,
                    source: text,
                    value,
                })
            }
        }
    }
}

impl LangValue for TomlString {
    fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    fn source(&self) -> &str {
        self.source.as_str()
    }
}

impl LangString for TomlString {
    fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl AsRef<str> for TomlString {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

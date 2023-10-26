use std::ops::Range;

use crate::lang::{LangBool, LangValue};

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TomlBool {
    pub(super) span: Range<usize>,
    pub(super) source: String,
    pub(super) value: bool,
}

impl TomlBool {
    pub(super) fn from_node(node: Node, source: impl AsRef<str>) -> Option<Self> {
        match node.as_bool() {
            None => None,
            Some(bool) => {
                let range = node.text_ranges().next().unwrap();
                let span = range_to_span(range);

                let source = source.as_ref();
                let text = source[span.clone()].to_string();

                let value = bool.value();
                Some(Self {
                    span,
                    source: text,
                    value,
                })
            }
        }
    }
}

impl LangValue for TomlBool {
    fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    fn source(&self) -> &str {
        self.source.as_str()
    }
}

impl LangBool for TomlBool {
    fn value(&self) -> bool {
        self.value
    }
}

impl AsRef<bool> for TomlBool {
    fn as_ref(&self) -> &bool {
        &self.value
    }
}

use std::ops::Range;

use crate::lang::{LangString, LangValue};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JsonString {
    pub(super) span: Range<usize>,
    pub(super) source: String,
    pub(super) value: String,
}

impl JsonString {
    pub(super) fn from_key(key: &str, range: Range<usize>, source: impl AsRef<str>) -> Self {
        let source = source.as_ref();
        let text = source[range.clone()].to_string();

        let value = key.to_string();
        Self {
            span: range,
            source: text,
            value,
        }
    }

    pub(super) fn from_node(
        node: Value<Span>,
        range: Range<usize>,
        source: impl AsRef<str>,
    ) -> Option<Self> {
        match node.as_str() {
            None => None,
            Some(string) => {
                let source = source.as_ref();
                let text = source[range.clone()].to_string();

                let value = string.to_string();
                Some(Self {
                    span: range,
                    source: text,
                    value,
                })
            }
        }
    }
}

impl LangValue for JsonString {
    fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    fn source(&self) -> &str {
        self.source.as_str()
    }
}

impl LangString for JsonString {
    fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl AsRef<str> for JsonString {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

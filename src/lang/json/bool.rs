use std::ops::Range;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct JsonBool {
    pub(super) span: Range<usize>,
    pub(super) source: String,
    pub(super) value: bool,
}

impl JsonBool {
    pub(super) fn from_node(
        node: Value<Span>,
        range: Range<usize>,
        source: impl AsRef<str>,
    ) -> Option<Self> {
        match node.as_boolean() {
            None => None,
            Some(bool) => {
                let source = source.as_ref();
                let text = source[range.clone()].to_string();

                Some(Self {
                    span: range,
                    source: text,
                    value: bool,
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

    pub fn value(&self) -> bool {
        self.value
    }
}

impl AsRef<bool> for JsonBool {
    fn as_ref(&self) -> &bool {
        &self.value
    }
}

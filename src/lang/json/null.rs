use std::ops::Range;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct JsonNull {
    pub(super) span: Range<usize>,
    pub(super) source: String,
}

impl JsonNull {
    pub(super) fn from_node(
        node: Value<Span>,
        range: Range<usize>,
        source: impl AsRef<str>,
    ) -> Option<Self> {
        if node.is_null() {
            let source = source.as_ref();
            let text = source[range.clone()].to_string();

            Some(Self {
                span: range,
                source: text,
            })
        } else {
            None
        }
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    pub fn value(&self) {}
}

impl AsRef<()> for JsonNull {
    fn as_ref(&self) -> &() {
        &()
    }
}

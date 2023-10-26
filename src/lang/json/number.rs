use std::ops::Range;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct JsonNumber {
    pub(super) span: Range<usize>,
    pub(super) source: String,
    pub(super) value: f64,
}

impl JsonNumber {
    pub(super) fn from_node(
        node: Value<Span>,
        range: Range<usize>,
        source: impl AsRef<str>,
    ) -> Option<Self> {
        match node.as_number() {
            None => None,
            Some(number) => {
                let source = source.as_ref();
                let text = source[range.clone()].to_string();

                let value = number.as_f64_lossy();
                Some(Self {
                    span: range,
                    source: text,
                    value,
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

    pub fn value(&self) -> f64 {
        self.value
    }
}

impl AsRef<f64> for JsonNumber {
    fn as_ref(&self) -> &f64 {
        &self.value
    }
}

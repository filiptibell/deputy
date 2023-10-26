use std::ops::Range;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct JsonArray {
    pub(super) span: Range<usize>,
    pub(super) source: String,
    pub(super) entries: Vec<JsonValue>,
}

impl JsonArray {
    pub(super) fn from_node(
        node: Value<Span>,
        range: Range<usize>,
        source: impl AsRef<str>,
    ) -> Option<Self> {
        match node.as_array() {
            None => None,
            Some(array) => {
                let source = source.as_ref();
                let text = source[range.clone()].to_string();

                let entries = array
                    .iter()
                    .map(|node| {
                        JsonValue::from_node(node.0.clone(), node.metadata().range(), source)
                    })
                    .collect();

                Some(Self {
                    span: range,
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

    pub fn entries(&self) -> &[JsonValue] {
        self.entries.as_ref()
    }
}

impl AsRef<[JsonValue]> for JsonArray {
    fn as_ref(&self) -> &[JsonValue] {
        &self.entries
    }
}

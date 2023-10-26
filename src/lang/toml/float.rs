use std::ops::Range;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TomlFloat {
    pub(super) span: Range<usize>,
    pub(super) source: String,
    pub(super) value: f64,
}

impl TomlFloat {
    pub(super) fn from_node(node: Node, source: impl AsRef<str>) -> Option<Self> {
        match node.as_float() {
            None => None,
            Some(string) => {
                let range = node.text_ranges().next().unwrap();
                let span = range_to_span(range);

                let source = source.as_ref();
                let text = source[span.clone()].to_string();

                let value = string.value();
                Some(Self {
                    span,
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

impl AsRef<f64> for TomlFloat {
    fn as_ref(&self) -> &f64 {
        &self.value
    }
}

use std::ops::Range;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TomlInteger {
    pub(super) span: Range<usize>,
    pub(super) source: String,
    pub(super) value: i64,
}

impl TomlInteger {
    pub(super) fn from_node(node: Node, source: impl AsRef<str>) -> Option<Self> {
        match node.as_integer() {
            None => None,
            Some(string) => {
                let range = node.text_ranges().next().unwrap();
                let span = range_to_span(range);

                let source = source.as_ref();
                let text = source[span.clone()].to_string();

                let value = match (string.value().as_positive(), string.value().as_negative()) {
                    (Some(pos), _) => pos as i64,
                    (_, Some(neg)) => neg,
                    _ => unreachable!(),
                };
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

    pub fn value(&self) -> i64 {
        self.value
    }
}

impl AsRef<i64> for TomlInteger {
    fn as_ref(&self) -> &i64 {
        &self.value
    }
}

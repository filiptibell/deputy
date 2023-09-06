use std::ops::{Deref, Range};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TomlInteger {
    pub(super) span: Range<usize>,
    pub(super) value: i64,
}

impl TomlInteger {
    pub(super) fn from_node(node: Node) -> Option<Self> {
        match node.as_integer() {
            None => None,
            Some(string) => {
                let range = node.text_ranges().next().unwrap();
                let span = range_to_span(range);
                let value = match (string.value().as_positive(), string.value().as_negative()) {
                    (Some(pos), _) => pos as i64,
                    (_, Some(neg)) => neg,
                    _ => unreachable!(),
                };
                Some(Self { span, value })
            }
        }
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

impl Deref for TomlInteger {
    type Target = i64;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

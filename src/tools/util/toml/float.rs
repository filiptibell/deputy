use std::ops::{Deref, Range};

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TomlFloat {
    pub(super) span: Range<usize>,
    pub(super) value: f64,
}

impl TomlFloat {
    pub(super) fn from_node(node: Node) -> Option<Self> {
        match node.as_float() {
            None => None,
            Some(string) => {
                let range = node.text_ranges().next().unwrap();
                let span = range_to_span(range);
                let value = string.value();
                Some(Self { span, value })
            }
        }
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

impl Deref for TomlFloat {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

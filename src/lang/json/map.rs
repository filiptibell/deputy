use std::{collections::HashMap, ops::Range};

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct JsonMap {
    pub(super) span: Range<usize>,
    pub(super) source: String,
    pub(super) entries: HashMap<JsonString, JsonValue>,
}

impl JsonMap {
    pub(super) fn from_node(
        node: Value<Span>,
        range: Range<usize>,
        source: impl AsRef<str>,
    ) -> Option<Self> {
        match node.as_object() {
            None => None,
            Some(object) => {
                let source = source.as_ref();
                let text = source[range.clone()].to_string();

                let entries = object
                    .entries()
                    .iter()
                    .map(|entry| {
                        (
                            JsonString::from_key(
                                entry.as_key().0.as_str(),
                                entry.key_metadata().range(),
                                source,
                            ),
                            JsonValue::from_node(
                                entry.as_value().0.clone(),
                                entry.value_metadata().range(),
                                source,
                            ),
                        )
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

    pub fn entries(&self) -> &HashMap<JsonString, JsonValue> {
        &self.entries
    }

    pub fn find(&self, key: impl AsRef<str>) -> Option<(&JsonString, &JsonValue)> {
        let key = key.as_ref();
        for (k, v) in &self.entries {
            if k.value.as_str() == key {
                return Some((k, v));
            }
        }
        None
    }
}

impl AsRef<HashMap<JsonString, JsonValue>> for JsonMap {
    fn as_ref(&self) -> &HashMap<JsonString, JsonValue> {
        &self.entries
    }
}

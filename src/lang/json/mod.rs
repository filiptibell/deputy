#![allow(dead_code)]
#![allow(unused_imports)]

use std::{fmt, ops::Range};

use json_syntax::{Kind, Parse, Value};
use locspan::Span;

mod array;
mod bool;
mod error;
mod map;
mod null;
mod number;
mod string;

pub use array::*;
pub use bool::*;
pub use error::*;
pub use map::*;
pub use null::*;
pub use number::*;
pub use string::*;

use super::LangValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonValueKind {
    Array,
    Bool,
    Map,
    Null,
    Number,
    String,
}

impl JsonValueKind {
    #[inline]
    pub const fn is_array(&self) -> bool {
        matches!(self, Self::Array)
    }

    #[inline]
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool)
    }

    #[inline]
    pub const fn is_map(&self) -> bool {
        matches!(self, Self::Map)
    }

    #[inline]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    #[inline]
    pub const fn is_number(&self) -> bool {
        matches!(self, Self::Number)
    }

    #[inline]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }
}

impl fmt::Display for JsonValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Array => "Array",
            Self::Bool => "Bool",
            Self::Map => "Table",
            Self::Null => "Null",
            Self::Number => "Number",
            Self::String => "String",
        };
        s.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Array(Box<JsonArray>),
    Bool(Box<JsonBool>),
    Map(Box<JsonMap>),
    Null(Box<JsonNull>),
    Number(Box<JsonNull>),
    String(Box<JsonString>),
}

impl JsonValue {
    fn from_node(node: Value<Span>, range: Range<usize>, source: impl AsRef<str>) -> Self {
        match node.kind() {
            Kind::Array => {
                Self::Array(Box::new(JsonArray::from_node(node, range, source).unwrap()))
            }
            Kind::Boolean => {
                Self::Bool(Box::new(JsonBool::from_node(node, range, source).unwrap()))
            }
            Kind::Null => Self::Null(Box::new(JsonNull::from_node(node, range, source).unwrap())),
            Kind::Number => {
                Self::Number(Box::new(JsonNull::from_node(node, range, source).unwrap()))
            }
            Kind::Object => Self::Map(Box::new(JsonMap::from_node(node, range, source).unwrap())),
            Kind::String => Self::String(Box::new(
                JsonString::from_node(node, range, source).unwrap(),
            )),
        }
    }

    pub fn new(source: impl AsRef<str>) -> Result<Self, JsonError> {
        let value = Value::parse_str(source.as_ref(), |s| s)
            .map_err(|e| JsonError::ParsingError(e.0.to_string()))?;
        Ok(Self::from_node(value.0, value.1.range(), source))
    }

    pub fn kind(&self) -> JsonValueKind {
        match self {
            Self::Array(_) => JsonValueKind::Array,
            Self::Bool(_) => JsonValueKind::Bool,
            Self::Map(_) => JsonValueKind::Map,
            Self::Null(_) => JsonValueKind::Null,
            Self::Number(_) => JsonValueKind::Number,
            Self::String(_) => JsonValueKind::String,
        }
    }

    #[inline]
    pub fn as_array(&self) -> Option<&JsonArray> {
        match self {
            Self::Array(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Option<&JsonBool> {
        match self {
            Self::Bool(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_map(&self) -> Option<&JsonMap> {
        match self {
            Self::Map(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_null(&self) -> Option<&JsonNull> {
        match self {
            Self::Null(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_number(&self) -> Option<&JsonNull> {
        match self {
            Self::Number(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_string(&self) -> Option<&JsonString> {
        match self {
            Self::String(v) => Some(v.as_ref()),
            _ => None,
        }
    }
}

impl LangValue for JsonValue {
    fn span(&self) -> Range<usize> {
        match self {
            Self::Array(v) => v.span(),
            Self::Bool(v) => v.span(),
            Self::Map(v) => v.span(),
            Self::Null(v) => v.span(),
            Self::Number(v) => v.span(),
            Self::String(v) => v.span(),
        }
    }

    fn source(&self) -> &str {
        match self {
            Self::Array(v) => v.source(),
            Self::Bool(v) => v.source(),
            Self::Map(v) => v.source(),
            Self::Null(v) => v.source(),
            Self::Number(v) => v.source(),
            Self::String(v) => v.source(),
        }
    }
}

#![allow(dead_code)]

use std::{fmt, ops::Range};

use taplo::{dom::Node, parser::parse, rowan::TextRange};

mod array;
mod bool;
mod error;
mod float;
mod integer;
mod string;
mod table;

pub use array::*;
pub use bool::*;
pub use error::*;
pub use float::*;
pub use integer::*;
pub use string::*;
pub use table::*;

fn range_to_span(range: TextRange) -> Range<usize> {
    Range {
        start: u32::from(range.start()) as usize,
        end: u32::from(range.end()) as usize,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TomlValueKind {
    Array,
    Bool,
    Float,
    Integer,
    String,
    Table,
}

impl TomlValueKind {
    #[inline]
    pub const fn is_array(&self) -> bool {
        matches!(self, Self::Array)
    }

    #[inline]
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool)
    }

    #[inline]
    pub const fn is_float(&self) -> bool {
        matches!(self, Self::Float)
    }

    #[inline]
    pub const fn is_integer(&self) -> bool {
        matches!(self, Self::Integer)
    }

    #[inline]
    pub const fn is_string(&self) -> bool {
        matches!(self, Self::String)
    }

    #[inline]
    pub const fn is_table(&self) -> bool {
        matches!(self, Self::Table)
    }
}

impl fmt::Display for TomlValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Array => "Array",
            Self::Bool => "Bool",
            Self::Float => "Float",
            Self::Integer => "Integer",
            Self::String => "String",
            Self::Table => "Table",
        };
        s.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TomlValue {
    Array(Box<TomlArray>),
    Bool(Box<TomlBool>),
    Float(Box<TomlFloat>),
    Integer(Box<TomlInteger>),
    String(Box<TomlString>),
    Table(Box<TomlTable>),
}

impl TomlValue {
    fn from_node(node: Node, source: impl AsRef<str>) -> Self {
        match node {
            Node::Array(_) => Self::Array(Box::new(TomlArray::from_node(node, source).unwrap())),
            Node::Bool(_) => Self::Bool(Box::new(TomlBool::from_node(node, source).unwrap())),
            Node::Float(_) => Self::Float(Box::new(TomlFloat::from_node(node, source).unwrap())),
            Node::Integer(_) => {
                Self::Integer(Box::new(TomlInteger::from_node(node, source).unwrap()))
            }
            Node::Str(_) => Self::String(Box::new(TomlString::from_node(node, source).unwrap())),
            Node::Table(_) => Self::Table(Box::new(TomlTable::from_node(node, source).unwrap())),
            _ => unimplemented!(),
        }
    }

    pub fn new(source: impl AsRef<str>) -> Result<Self, TomlError> {
        let mut parsed = parse(source.as_ref());
        if let Some(e) = parsed.errors.pop() {
            return Err(e.into());
        }

        let root = parsed.into_dom();
        if let Err(mut e) = root.validate() {
            let e = e.next().unwrap();
            return Err(e.into());
        }

        Ok(TomlValue::from_node(root, source))
    }

    pub fn kind(&self) -> TomlValueKind {
        match self {
            Self::Array(_) => TomlValueKind::Array,
            Self::Bool(_) => TomlValueKind::Bool,
            Self::Float(_) => TomlValueKind::Float,
            Self::Integer(_) => TomlValueKind::Integer,
            Self::String(_) => TomlValueKind::String,
            Self::Table(_) => TomlValueKind::Table,
        }
    }

    #[inline]
    pub fn as_array(&self) -> Option<&TomlArray> {
        match self {
            Self::Array(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Option<&TomlBool> {
        match self {
            Self::Bool(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_float(&self) -> Option<&TomlFloat> {
        match self {
            Self::Float(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_integer(&self) -> Option<&TomlInteger> {
        match self {
            Self::Integer(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_string(&self) -> Option<&TomlString> {
        match self {
            Self::String(v) => Some(v.as_ref()),
            _ => None,
        }
    }

    #[inline]
    pub fn as_table(&self) -> Option<&TomlTable> {
        match self {
            Self::Table(v) => Some(v.as_ref()),
            _ => None,
        }
    }
}

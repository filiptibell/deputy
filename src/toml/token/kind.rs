#![allow(dead_code)]

use std::fmt;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Comment,
    Float,
    Integer,
    String,
    Symbol,
    Whitespace,
}

impl TokenKind {
    #[inline]
    pub const fn is_comment(&self) -> bool {
        matches!(self, Self::Comment)
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
    pub const fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol)
    }

    #[inline]
    pub const fn is_whitespace(&self) -> bool {
        matches!(self, Self::Whitespace)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Comment => "Comment",
            Self::Float => "Float",
            Self::Integer => "Integer",
            Self::String => "String",
            Self::Symbol => "Symbol",
            Self::Whitespace => "Whitespace",
        };
        name.fmt(f)
    }
}

impl From<&TokenValue<'_>> for TokenKind {
    fn from(value: &TokenValue) -> Self {
        match value {
            TokenValue::Comment(_) => Self::Comment,
            TokenValue::Float(_) => Self::Float,
            TokenValue::Integer(_) => Self::Integer,
            TokenValue::String(_) => Self::String,
            TokenValue::Symbol(_) => Self::Symbol,
            TokenValue::Whitespace(_) => Self::Whitespace,
        }
    }
}

#![allow(dead_code)]

use std::{borrow::Cow, fmt};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl From<RawToken<'_>> for TokenKind {
    fn from(value: RawToken<'_>) -> Self {
        match value {
            RawToken::Comment(_) => Self::Comment,
            RawToken::Dot => Self::Symbol,
            RawToken::Equals => Self::Symbol,
            RawToken::Integer(_) => Self::Integer,
            RawToken::LeftBracket => Self::Symbol,
            RawToken::LeftBrace => Self::Symbol,
            RawToken::RightBracket => Self::Symbol,
            RawToken::RightBrace => Self::Symbol,
            RawToken::String(_) => Self::String,
            RawToken::Whitespace(_) => Self::Whitespace,
        }
    }
}

impl From<&RawToken<'_>> for TokenKind {
    fn from(value: &RawToken) -> Self {
        value.into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue<'a> {
    Comment(Cow<'a, str>),
    Float(f64),
    Integer(u64),
    String(Cow<'a, str>),
    Symbol(&'static str),
    Whitespace(Cow<'a, str>),
}

impl<'a> TokenValue<'a> {
    #[inline]
    pub const fn kind(&self) -> TokenKind {
        match self {
            Self::Comment(_) => TokenKind::Comment,
            Self::Float(_) => TokenKind::Float,
            Self::Integer(_) => TokenKind::Integer,
            Self::String(_) => TokenKind::String,
            Self::Symbol(_) => TokenKind::Symbol,
            Self::Whitespace(_) => TokenKind::Whitespace,
        }
    }

    #[inline]
    pub fn as_comment(&self) -> &str {
        match self {
            Self::Comment(s) => s,
            v => panic!("token kind mismatch - expected Comment, got {}", v.kind()),
        }
    }

    #[inline]
    pub fn as_float(&self) -> f64 {
        match self {
            Self::Float(f) => *f,
            v => panic!("token kind mismatch - expected Float, got {}", v.kind()),
        }
    }

    #[inline]
    pub fn as_integer(&self) -> u64 {
        match self {
            Self::Integer(u) => *u,
            v => panic!("token kind mismatch - expected Integer, got {}", v.kind()),
        }
    }

    #[inline]
    pub fn as_string(&self) -> &str {
        match self {
            Self::String(s) => s,
            v => panic!("token kind mismatch - expected String, got {}", v.kind()),
        }
    }

    #[inline]
    pub fn as_symbol(&self) -> &'static str {
        match self {
            Self::Symbol(s) => s,
            v => panic!("token kind mismatch - expected Symbol, got {}", v.kind()),
        }
    }

    #[inline]
    pub fn as_whitespace(&self) -> &str {
        match self {
            Self::Whitespace(s) => s,
            v => panic!(
                "token kind mismatch - expected Whitespace, got {}",
                v.kind()
            ),
        }
    }
}

impl<'a> From<RawToken<'a>> for TokenValue<'a> {
    fn from(value: RawToken<'a>) -> Self {
        match value {
            RawToken::Comment(c) => Self::Comment(Cow::Borrowed(c)),
            RawToken::Dot => Self::Symbol("."),
            RawToken::Equals => Self::Symbol("="),
            RawToken::Integer(u) => Self::Integer(u),
            RawToken::LeftBracket => Self::Symbol("["),
            RawToken::LeftBrace => Self::Symbol("{"),
            RawToken::RightBracket => Self::Symbol("]"),
            RawToken::RightBrace => Self::Symbol("}"),
            RawToken::String(s) => Self::String(Cow::Borrowed(s)),
            RawToken::Whitespace(s) => Self::Whitespace(Cow::Borrowed(s)),
        }
    }
}

impl<'a> From<&'a RawToken<'a>> for TokenValue<'a> {
    fn from(value: &'a RawToken) -> Self {
        value.into()
    }
}

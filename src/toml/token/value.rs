#![allow(dead_code)]

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue<'a> {
    Symbol(&'static str),
    String(Cow<'a, str>),
    Integer(u64),
    Float(f64),
    Whitespace(Cow<'a, str>),
}

impl<'a> TokenValue<'a> {
    #[inline]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Symbol(_) => "Symbol",
            Self::String(_) => "String",
            Self::Integer(_) => "Integer",
            Self::Float(_) => "Float",
            Self::Whitespace(_) => "Whitespace",
        }
    }

    #[inline]
    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(_))
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    #[inline]
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(_))
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    #[inline]
    pub fn is_whitespace(&self) -> bool {
        matches!(self, Self::Whitespace(_))
    }

    #[inline]
    pub fn as_symbol(&self) -> &'static str {
        match self {
            Self::Symbol(s) => s,
            v => panic!("token kind mismatch - expected Symbol, got {:?}", v.kind()),
        }
    }

    #[inline]
    pub fn as_string(&self) -> &str {
        match self {
            Self::String(s) => s,
            v => panic!("token kind mismatch - expected String, got {:?}", v.kind()),
        }
    }

    #[inline]
    pub fn as_integer(&self) -> u64 {
        match self {
            Self::Integer(u) => *u,
            v => panic!("token kind mismatch - expected Integer, got {:?}", v.kind()),
        }
    }

    #[inline]
    pub fn as_float(&self) -> f64 {
        match self {
            Self::Float(f) => *f,
            v => panic!("token kind mismatch - expected Float, got {:?}", v.kind()),
        }
    }

    #[inline]
    pub fn as_whitespace(&self) -> &str {
        match self {
            Self::Whitespace(s) => s,
            v => panic!(
                "token kind mismatch - expected Whitespace, got {:?}",
                v.kind()
            ),
        }
    }
}

impl<'a> From<RawToken<'a>> for TokenValue<'a> {
    fn from(value: RawToken<'a>) -> Self {
        match value {
            RawToken::Dot => Self::Symbol("."),
            RawToken::Equals => Self::Symbol("="),
            RawToken::LeftBracket => Self::Symbol("["),
            RawToken::LeftBrace => Self::Symbol("{"),
            RawToken::RightBracket => Self::Symbol("]"),
            RawToken::RightBrace => Self::Symbol("}"),
            RawToken::Comment(c) => Self::String(Cow::Borrowed(c)),
            RawToken::String(s) => Self::String(Cow::Borrowed(s)),
            RawToken::Integer(u) => Self::Integer(u),
            RawToken::Whitespace(s) => Self::Whitespace(Cow::Borrowed(s)),
        }
    }
}

impl<'a> From<&'a RawToken<'a>> for TokenValue<'a> {
    fn from(value: &'a RawToken) -> Self {
        value.into()
    }
}

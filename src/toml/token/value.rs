#![allow(dead_code)]

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Static(&'static str),
    String(String),
    Integer(u64),
    Float(f64),
}

impl TokenValue {
    pub fn as_static(&self) -> &'static str {
        match self {
            Self::Static(s) => s,
            v => panic!("token value is not a static, was {v:?}"),
        }
    }

    pub fn as_string(&self) -> &str {
        match self {
            Self::String(s) => s,
            v => panic!("token value is not a string, was {v:?}"),
        }
    }

    pub fn as_integer(&self) -> u64 {
        match self {
            Self::Integer(u) => *u,
            v => panic!("token value is not an integer, was {v:?}"),
        }
    }

    pub fn as_float(&self) -> f64 {
        match self {
            Self::Float(f) => *f,
            v => panic!("token value is not a number, was {v:?}"),
        }
    }
}

impl From<RawToken<'_>> for TokenValue {
    fn from(value: RawToken) -> Self {
        match value {
            RawToken::Dot => Self::Static("."),
            RawToken::Equals => Self::Static("="),
            RawToken::LeftBracket => Self::Static("["),
            RawToken::LeftBrace => Self::Static("{"),
            RawToken::RightBracket => Self::Static("]"),
            RawToken::RightBrace => Self::Static("}"),
            RawToken::Comment(c) => Self::String(c.to_string()),
            RawToken::String(s) => Self::String(s.to_string()),
            RawToken::Integer(u) => Self::Integer(u),
            RawToken::Whitespace => panic!("Whitespace should be skipped"),
        }
    }
}

impl From<&RawToken<'_>> for TokenValue {
    fn from(value: &RawToken) -> Self {
        value.into()
    }
}

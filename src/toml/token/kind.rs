#![allow(dead_code)]

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Dot,
    Equals,
    LeftBracket,
    LeftBrace,
    RightBracket,
    RightBrace,
    Comment,
    String,
    Integer,
}

impl From<RawToken<'_>> for TokenKind {
    fn from(value: RawToken) -> Self {
        match value {
            RawToken::Dot => Self::Dot,
            RawToken::Equals => Self::Equals,
            RawToken::LeftBracket => Self::LeftBracket,
            RawToken::LeftBrace => Self::LeftBrace,
            RawToken::RightBracket => Self::RightBracket,
            RawToken::RightBrace => Self::RightBrace,
            RawToken::Comment(_) => Self::Comment,
            RawToken::String(_) => Self::String,
            RawToken::Integer(_) => Self::Integer,
            RawToken::Whitespace => panic!("Whitespace should be skipped"),
        }
    }
}

impl From<&RawToken<'_>> for TokenKind {
    fn from(value: &RawToken) -> Self {
        value.into()
    }
}

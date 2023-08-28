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

impl TokenKind {
    pub fn is_dot(&self) -> bool {
        *self == Self::Dot
    }

    pub fn is_equals(&self) -> bool {
        *self == Self::Equals
    }

    pub fn is_left_bracket(&self) -> bool {
        *self == Self::LeftBracket
    }

    pub fn is_left_brace(&self) -> bool {
        *self == Self::LeftBrace
    }

    pub fn is_right_bracket(&self) -> bool {
        *self == Self::RightBracket
    }

    pub fn is_right_brace(&self) -> bool {
        *self == Self::RightBrace
    }

    pub fn is_comment(&self) -> bool {
        *self == Self::Comment
    }

    pub fn is_string(&self) -> bool {
        *self == Self::String
    }

    pub fn is_integer(&self) -> bool {
        *self == Self::Integer
    }
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

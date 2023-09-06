use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Dot,
    Equals,
    LeftBracket,
    LeftBrace,
    RightBracket,
    RightBrace,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Dot => ".",
            Self::Equals => "=",
            Self::LeftBracket => "[",
            Self::LeftBrace => "{",
            Self::RightBracket => "]",
            Self::RightBrace => "}",
        };
        s.fmt(f)
    }
}

impl FromStr for Symbol {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "." => Self::Dot,
            "=" => Self::Equals,
            "[" => Self::LeftBracket,
            "{" => Self::LeftBrace,
            "]" => Self::RightBracket,
            "}" => Self::RightBrace,
            v => return Err(format!("Unknown symbol '{v}'")),
        })
    }
}

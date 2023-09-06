#![allow(dead_code)]

use logos::{Lexer, Logos};

use super::*;

fn parse_float<'a>(lex: &mut Lexer<'a, TokenValue<'a>>) -> TokenizerResult<f64> {
    Ok(lex.slice().parse()?)
}

fn parse_integer<'a>(
    lex: &mut Lexer<'a, TokenValue<'a>>,
    prefix: &'static str,
    radix: u32,
) -> TokenizerResult<u64> {
    let slice = lex.slice();
    let no_prefix = &slice[prefix.len()..];
    if no_prefix.chars().any(|c| c == '_') {
        let no_separator = no_prefix.chars().filter(|c| c != &'_').collect::<String>();
        Ok(u64::from_str_radix(&no_separator, radix)?)
    } else {
        Ok(u64::from_str_radix(no_prefix, radix)?)
    }
}

fn read_string_basic<'a>(lex: &mut Lexer<'a, TokenValue<'a>>) -> TokenizerResult<&'a str> {
    let mut escape = false;
    for char in lex.remainder().chars() {
        match (escape, char) {
            (true, ..) => escape = false,
            (false, '\\') => escape = true,
            (false, '\n' | '\r') => break,
            (false, ..) if char == '"' => {
                lex.bump(1);
                return Ok(&lex.source()[lex.span()]);
            }
            _ => {}
        }
        lex.bump(char.len_utf8());
    }
    Err(TokenizerError::IncompleteStringBasic)
}

fn read_string_literal<'a>(lex: &mut Lexer<'a, TokenValue<'a>>) -> TokenizerResult<&'a str> {
    for char in lex.remainder().chars() {
        match char {
            '\n' | '\r' => break,
            '\'' => {
                lex.bump(1);
                return Ok(&lex.source()[lex.span()]);
            }
            _ => lex.bump(char.len_utf8()),
        }
    }
    Err(TokenizerError::IncompleteStringLiteral)
}

fn read_comment<'a>(lex: &mut Lexer<'a, TokenValue<'a>>) -> TokenizerResult<&'a str> {
    for char in lex.remainder().chars() {
        if matches!(char, '\n' | '\r') {
            break;
        }
        lex.bump(char.len_utf8());
    }
    Ok(&lex.source()[lex.span()])
}

fn read_symbol<'a>(lex: &mut Lexer<'a, TokenValue<'a>>) -> TokenizerResult<Symbol> {
    Ok(lex.slice().parse().expect("Unimplemented symbol"))
}

#[derive(Logos, Debug, Clone, Copy, PartialEq)]
#[logos(error = TokenizerError)]
pub enum TokenValue<'a> {
    #[regex(r"#.*", read_comment)]
    Comment(&'a str),

    #[token(".", read_symbol)]
    #[token("=", read_symbol)]
    #[token("[", read_symbol)]
    #[token("]", read_symbol)]
    #[token("{", read_symbol)]
    #[token("}", read_symbol)]
    Symbol(Symbol),

    #[regex(r"[-+]?\d+(\.\d+)?", parse_float)]
    Float(f64),

    #[regex(r"0x[a-fA-F0-9_]+", |lex| parse_integer(lex, "0x", 16))]
    #[regex(r"0o[0-8_]+", |lex| parse_integer(lex, "0o", 8))]
    #[regex(r"0b[01_]+", |lex| parse_integer(lex, "0b", 2))]
    Integer(u64),

    #[token("'", read_string_literal)]
    #[token("\"", read_string_basic)]
    #[regex(r"[a-zA-Z][a-zA-Z0-9\-_]*")]
    String(&'a str),

    #[regex(r"[ \t\n\f]+")]
    Whitespace(&'a str),
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
    pub fn as_symbol(&self) -> Symbol {
        match self {
            Self::Symbol(s) => *s,
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

impl std::hash::Hash for TokenValue<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Comment(c) => c.hash(state),
            Self::Float(f) => f.to_string().hash(state),
            Self::Integer(u) => u.hash(state),
            Self::String(s) => s.hash(state),
            Self::Symbol(s) => s.hash(state),
            Self::Whitespace(s) => s.hash(state),
        }
    }
}

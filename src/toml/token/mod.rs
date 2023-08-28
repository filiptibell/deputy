use std::ops::Range;

use logos::Logos;

use super::error::*;

mod raw;
use raw::*;

mod kind;
mod value;

pub use kind::*;
pub use value::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub span: Range<usize>,
    pub kind: TokenKind,
    pub value: TokenValue,
    pub text: String,
}

impl Token {
    pub fn parse_all(source: impl AsRef<str>) -> ParserResult<Vec<Token>> {
        let source = source.as_ref();
        let spanned = RawToken::lexer(source).spanned().collect::<Vec<_>>();

        let mut results = Vec::new();

        for (tok, span) in spanned {
            // NOTE: Doing &tok? here instead of tok? causes stack overflow ???
            let tok = tok?;
            results.push(Self {
                span: span.clone(),
                kind: tok.into(),
                value: tok.into(),
                text: source[span].to_string(),
            });
        }

        Ok(results)
    }
}

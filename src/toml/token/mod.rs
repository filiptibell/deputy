use std::{borrow::Cow, ops::Range};

use logos::Logos;

mod raw;
use raw::*;

mod error;
mod value;

pub use error::*;
pub use value::*;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub span: Range<usize>,
    pub value: TokenValue<'a>,
    pub text: Cow<'a, str>,
}

#[derive(Debug, Clone)]
pub struct Tokenizer;

impl Tokenizer {
    pub fn parse(source: &str) -> TokenizerResult<Vec<Token>> {
        let spanned = RawToken::lexer(source).spanned().collect::<Vec<_>>();

        let mut results = Vec::new();

        for (tok, span) in spanned {
            // NOTE: Doing &tok? here instead of tok? causes stack overflow ???
            let tok = tok?;
            results.push(Token {
                span: span.clone(),
                value: tok.into(),
                text: Cow::Borrowed(&source[span]),
            });
        }

        Ok(results)
    }

    pub fn parse_ignore_whitespace(source: &str) -> TokenizerResult<Vec<Token>> {
        let tokens = Self::parse(source)?
            .into_iter()
            .filter(|t| !t.value.is_whitespace())
            .collect();
        Ok(tokens)
    }
}

use std::ops::Range;

use logos::{Lexer, Logos};

mod raw;
use raw::*;

mod error;
mod value;

pub use error::*;
pub use value::*;

#[cfg(test)]
mod tests;

#[derive(Clone, PartialEq)]
pub struct Token<'a> {
    pub span: Range<usize>,
    pub kind: TokenKind,
    pub value: TokenValue<'a>,
}

pub struct Tokenizer<'a> {
    lexer: Lexer<'a, RawToken<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: RawToken::lexer(source),
        }
    }

    pub fn next(&mut self) -> TokenizerResult<Option<Token>> {
        if let Some(res) = self.lexer.next() {
            let raw = res?;
            let span = self.lexer.span();
            Ok(Some(Token {
                span: span.clone(),
                kind: raw.into(),
                value: raw.into(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn parse_all(mut self) -> TokenizerResult<Vec<Token<'a>>> {
        let mut tokens = Vec::new();
        while let Some(res) = self.lexer.next() {
            let raw = res?;
            let span = self.lexer.span();
            tokens.push(Token {
                span: span.clone(),
                kind: raw.into(),
                value: raw.into(),
            })
        }
        Ok(tokens)
    }
}

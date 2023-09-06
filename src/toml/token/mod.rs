use std::ops::Range;

use logos::Logos;

mod error;
mod kind;
mod symbol;
mod value;

pub use error::*;
pub use kind::*;
pub use symbol::*;
pub use value::*;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Token<'a> {
    pub span: Range<usize>,
    pub kind: TokenKind,
    pub value: TokenValue<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tokenizer;

impl Tokenizer {
    pub fn parse(source: &str) -> TokenizerResult<Vec<Token>> {
        TokenValue::lexer(source)
            .spanned()
            .map(|(res, span)| {
                res.map(|value| Token {
                    span,
                    kind: (&value).into(),
                    value,
                })
            })
            .collect()
    }
}

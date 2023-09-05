use std::vec::IntoIter;

use super::*;

mod integers;
mod suites;

fn iter_no_whitespace(source: &'static str) -> IntoIter<Token> {
    let tokens = Tokenizer::parse_ignore_whitespace(source)
        .expect("Failed to parse")
        .into_iter();
    tokens
}

fn next_val<'a>(tokens: &mut IntoIter<Token<'a>>) -> TokenValue<'a> {
    tokens.next().unwrap().value
}

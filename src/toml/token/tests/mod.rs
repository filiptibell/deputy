use std::vec::IntoIter;

use super::*;

mod integers;
mod suites;

fn iter_no_whitespace(source: &'static str) -> IntoIter<Token> {
    Tokenizer::new(source)
        .parse_all()
        .expect("Failed to parse")
        .into_iter()
        .filter(|t| !t.kind.is_whitespace())
        .collect::<Vec<_>>()
        .into_iter()
}

fn next_val<'a>(tokens: &mut IntoIter<Token<'a>>) -> TokenValue<'a> {
    tokens.next().unwrap().value
}

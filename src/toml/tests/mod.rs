use std::vec::IntoIter;

use crate::toml::*;

mod integers;
mod suites;

fn next_val(tokens: &mut IntoIter<Token>) -> TokenValue {
    tokens.next().unwrap().value
}

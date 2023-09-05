use logos::{Lexer, Logos};

use super::*;

fn parse_integer<'a>(
    lex: &mut Lexer<'a, RawToken<'a>>,
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

fn read_string_basic<'a>(lex: &mut Lexer<'a, RawToken<'a>>) -> TokenizerResult<&'a str> {
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

fn read_string_literal<'a>(lex: &mut Lexer<'a, RawToken<'a>>) -> TokenizerResult<&'a str> {
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

fn read_comment<'a>(lex: &mut Lexer<'a, RawToken<'a>>) -> TokenizerResult<&'a str> {
    for char in lex.remainder().chars() {
        if matches!(char, '\n' | '\r') {
            break;
        }
        lex.bump(char.len_utf8());
    }
    Ok(&lex.source()[lex.span()])
}

#[doc(hidden)]
#[derive(Logos, Debug, Clone, PartialEq, Eq, Copy)]
#[logos(error = TokenizerError)]
pub enum RawToken<'a> {
    #[regex(r"#.*", read_comment)]
    Comment(&'a str),

    #[token(".")]
    Dot,

    #[token("=")]
    Equals,

    #[regex(r"0x[a-fA-F0-9_]+", |lex| parse_integer(lex, "0x", 16))]
    #[regex(r"0o[0-8_]+", |lex| parse_integer(lex, "0o", 8))]
    #[regex(r"0b[01_]+", |lex| parse_integer(lex, "0b", 2))]
    Integer(u64),

    #[token("[")]
    LeftBracket,

    #[token("{")]
    LeftBrace,

    #[token("]")]
    RightBracket,

    #[token("}")]
    RightBrace,

    #[token("'", read_string_literal)]
    #[token("\"", read_string_basic)]
    #[regex(r"[a-zA-Z0-9\-_]+")]
    String(&'a str),

    #[regex(r"[ \t\n\f]+")]
    Whitespace(&'a str),
}

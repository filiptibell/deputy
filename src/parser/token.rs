use logos::{Lexer, Logos};

fn parse_uint(lex: &mut Lexer<Token>, prefix: &'static str, radix: u32) -> Option<u64> {
    let slice = lex.slice();
    let no_prefix = &slice[prefix.len()..];
    if no_prefix.chars().any(|c| c == '_') {
        let no_separator = no_prefix.chars().filter(|c| c != &'_').collect::<String>();
        u64::from_str_radix(&no_separator, radix).ok()
    } else {
        u64::from_str_radix(no_prefix, radix).ok()
    }
}

fn read_string_basic(lex: &mut Lexer<Token>) -> bool {
    let mut escape = false;
    for char in lex.remainder().chars() {
        match (escape, char) {
            (true, ..) => escape = false,
            (false, '\\') => escape = true,
            (false, '\n' | '\r') => break,
            (false, ..) if char == '"' => {
                lex.bump(1);
                return true;
            }
            _ => {}
        }
        lex.bump(char.len_utf8());
    }
    false
}

fn read_string_literal(lex: &mut Lexer<Token>) -> bool {
    for char in lex.remainder().chars() {
        match char {
            '\n' | '\r' => break,
            '\'' => {
                lex.bump(1);
                return true;
            }
            _ => lex.bump(char.len_utf8()),
        }
    }
    false
}

fn read_comment(lex: &mut Lexer<Token>) -> bool {
    let mut current_offset = 0;
    for (offset, char) in lex.remainder().char_indices() {
        if matches!(char, '\n' | '\r') {
            lex.bump(offset);
            return true;
        }
        current_offset = offset + char.len_utf8();
    }
    lex.bump(current_offset);
    true
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("[")]
    LeftBracket,

    #[token("{")]
    LeftBrace,

    #[token("]")]
    RightBracket,

    #[token("}")]
    RightBrace,

    #[token("=")]
    Assignment,

    #[token(".")]
    Dot,

    #[regex(r"[a-zA-Z0-9\-_]+")]
    Key,

    #[regex("0x[a-fA-F0-9_]+", |lex| parse_uint(lex, "0x", 16))]
    #[regex("0o[0-8_]+", |lex| parse_uint(lex, "0o", 8))]
    #[regex("0b[01_]+", |lex| parse_uint(lex, "0b", 2))]
    IntegerUnsigned(u64),

    #[token("'", read_string_literal)]
    StringLiteral,

    #[token("\"", read_string_basic)]
    StringBasic,

    #[regex(r"#.*", read_comment)]
    Comment,
}

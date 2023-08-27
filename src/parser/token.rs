use logos::{Lexer, Logos};

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
        if char == '\n' {
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
    Equals,

    #[regex(r"[a-zA-Z][a-zA-Z0-9\-_]*")]
    Identifier,

    #[token("'", read_string_literal)]
    StringLiteral,

    #[token("\"", read_string_basic)]
    StringBasic,

    #[regex(r"#.*", read_comment)]
    Comment,
}

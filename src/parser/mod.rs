use logos::{Lexer, Logos};

#[cfg(test)]
mod tests;

fn read_string(lex: &mut Lexer<Token>, quote: char) -> bool {
    let mut escape = false;
    for char in lex.remainder().chars() {
        match (escape, char) {
            (true, ..) => escape = false,
            (false, '\\') => escape = true,
            (false, '\n' | '\r') => break,
            (false, ..) if char == quote => {
                lex.bump(1);
                return true;
            }
            _ => {}
        }
        lex.bump(char.len_utf8());
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

    #[regex(r"'", |x| read_string(x, '\''))]
    SingleQuoteString,

    #[regex(r#"""#, |x| read_string(x, '"'))]
    DoubleQuoteString,

    #[regex(r"#.*", read_comment)]
    Comment,
}

use crate::toml::*;

const PREFIXED_NUMBERS: &str = r#"
hex1 = 0xDEADBEEF
hex2 = 0xdeadbeef
hex3 = 0xdead_beef

oct1 = 0o01234567
oct2 = 0o755

bin1 = 0b11010110
"#;

const SECTIONED: &str = r#"
[section]
key = "value" # A comment
1234 = 'numbers-and-literals'
"#;

#[test]
fn tokens_prefixed_numbers() {
    let mut lex = Token::lexer(PREFIXED_NUMBERS);

    let mut check = || {
        assert_eq!(lex.next(), Some(Ok(Token::Key)));
        assert_eq!(lex.next(), Some(Ok(Token::Assignment)));
        assert!(matches!(lex.next(), Some(Ok(Token::IntegerUnsigned(_)))));
    };

    for _ in 0..6 {
        check();
    }

    assert_eq!(lex.next(), None);
}

#[test]
fn tokens_sectioned() {
    let mut lex = Token::lexer(SECTIONED);

    assert_eq!(lex.next(), Some(Ok(Token::LeftBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::Key)));
    assert_eq!(lex.next(), Some(Ok(Token::RightBracket)));

    assert_eq!(lex.next(), Some(Ok(Token::Key)));
    assert_eq!(lex.next(), Some(Ok(Token::Assignment)));
    assert_eq!(lex.next(), Some(Ok(Token::String)));
    assert_eq!(lex.next(), Some(Ok(Token::Comment)));

    assert_eq!(lex.next(), Some(Ok(Token::Key)));
    assert_eq!(lex.next(), Some(Ok(Token::Assignment)));
    assert_eq!(lex.next(), Some(Ok(Token::String)));

    assert_eq!(lex.next(), None);
}

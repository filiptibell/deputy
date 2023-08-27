use logos::Logos;

use super::token::Token;

#[test]
fn parse_uint() {
    let mut lex = Token::lexer(
        r#"
        hex1 = 0xDEADBEEF
        hex2 = 0xdeadbeef
        hex3 = 0xdead_beef

        oct1 = 0o01234567
        oct2 = 0o755

        bin1 = 0b11010110
        "#,
    );

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
fn parse_manifest() {
    let mut lex = Token::lexer(
        r#"
        # Tools managed by the tool manager
        [tools]
        tool-name = "scope/name@1.2.3" # A comment
        super_alpha_rc = 'scope/name@0.0.1-alpha.rc.1'
        "#,
    );

    assert_eq!(lex.next(), Some(Ok(Token::Comment)));

    assert_eq!(lex.next(), Some(Ok(Token::LeftBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::Key)));
    assert_eq!(lex.next(), Some(Ok(Token::RightBracket)));

    assert_eq!(lex.next(), Some(Ok(Token::Key)));
    assert_eq!(lex.next(), Some(Ok(Token::Assignment)));
    assert_eq!(lex.next(), Some(Ok(Token::StringBasic)));
    assert_eq!(lex.next(), Some(Ok(Token::Comment)));

    assert_eq!(lex.next(), Some(Ok(Token::Key)));
    assert_eq!(lex.next(), Some(Ok(Token::Assignment)));
    assert_eq!(lex.next(), Some(Ok(Token::StringLiteral)));

    assert_eq!(lex.next(), None);
}

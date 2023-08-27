use logos::Logos;

use super::Token;

#[test]
pub fn parse_header() {
    let mut lex = Token::lexer(
        r#"
        # Tools managed by the tool manager
        [tools]
        "#,
    );

    assert_eq!(lex.next(), Some(Ok(Token::Comment)));
    assert_eq!(lex.next(), Some(Ok(Token::LeftBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::Identifier)));
    assert_eq!(lex.next(), Some(Ok(Token::RightBracket)));
    assert_eq!(lex.next(), None);
}

#[test]
pub fn parse_manifest() {
    let mut lex = Token::lexer(
        r#"
        [tools] # A comment
        tool-name = "scope/name@1.2.3"
        super_alpha_rc = "scope/name@0.0.1-alpha.rc.1"
        "#,
    );

    assert_eq!(lex.next(), Some(Ok(Token::LeftBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::Identifier)));
    assert_eq!(lex.next(), Some(Ok(Token::RightBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::Comment)));
    assert_eq!(lex.next(), Some(Ok(Token::Identifier)));
    assert_eq!(lex.next(), Some(Ok(Token::Equals)));
    assert_eq!(lex.next(), Some(Ok(Token::DoubleQuoteString)));
    assert_eq!(lex.next(), Some(Ok(Token::Identifier)));
    assert_eq!(lex.next(), Some(Ok(Token::Equals)));
    assert_eq!(lex.next(), Some(Ok(Token::DoubleQuoteString)));
    assert_eq!(lex.next(), None);
}

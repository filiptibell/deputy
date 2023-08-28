use logos::Logos;

use super::token::Token;
use super::Manifest;

// TODO: Move toml tests into toml module in root
const PREFIXED_NUMBERS: &str = r#"
hex1 = 0xDEADBEEF
hex2 = 0xdeadbeef
hex3 = 0xdead_beef

oct1 = 0o01234567
oct2 = 0o755

bin1 = 0b11010110
"#;

const MANIFEST: &str = r#"
[tools]
tool-name = "scope/name@1.2.3" # A comment
super_alpha_rc = 'scope/name@0.0.1-alpha.rc.1'
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
fn tokens_manifest() {
    let mut lex = Token::lexer(MANIFEST);

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

#[test]
fn parse_manifest() {
    let manifest = Manifest::parse(MANIFEST.trim());

    let manifest = manifest.expect("Failed to parse manifest");

    println!("Manifest: {manifest:#?}");

    assert_eq!(manifest.tools_header.span.start, 0);
    assert_eq!(manifest.tools_header.span.end, 7);

    assert_eq!(manifest.tools_map.tools.len(), 2);

    let first_tool = &manifest.tools_map.tools[0];
    assert_eq!(first_tool.key_text, r#"tool-name"#);
    assert_eq!(first_tool.val_text, r#""scope/name@1.2.3""#);

    let second_tool = &manifest.tools_map.tools[1];
    assert_eq!(second_tool.key_text, r#"super_alpha_rc"#);
    assert_eq!(second_tool.val_text, r#"'scope/name@0.0.1-alpha.rc.1'"#);
}

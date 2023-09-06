use super::*;

const SECTIONS: &str = r#"
[section]
key = "value" # A comment
1234 = 'number-strings-and-literals'

[another.empty.'section']
"#;

#[test]
fn sections() {
    let tokens = &mut iter_no_whitespace(SECTIONS);

    assert_eq!(next_val(tokens).as_symbol(), Symbol::LeftBracket);
    assert_eq!(next_val(tokens).as_string(), "section");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::RightBracket);

    assert_eq!(next_val(tokens).as_string(), "key");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::Equals);
    assert_eq!(next_val(tokens).as_string(), "\"value\"");
    assert_eq!(next_val(tokens).as_comment(), "# A comment");

    assert_eq!(next_val(tokens).as_string(), "1234");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::Equals);
    assert_eq!(
        next_val(tokens).as_string(),
        "'number-strings-and-literals'"
    );

    assert_eq!(next_val(tokens).as_symbol(), Symbol::LeftBracket);
    assert_eq!(next_val(tokens).as_string(), "another");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::Dot);
    assert_eq!(next_val(tokens).as_string(), "empty");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::Dot);
    assert_eq!(next_val(tokens).as_string(), "'section'");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::RightBracket);

    assert!(tokens.next().is_none());
}

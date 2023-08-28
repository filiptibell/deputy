use super::*;

const SECTIONED: &str = r#"
[section]
key = "value" # A comment
1234 = 'number-strings-and-literals'
"#;

#[test]
fn sectioned() {
    let tokens = &mut Token::parse_all(SECTIONED)
        .expect("Failed to parse")
        .into_iter();

    assert_eq!(next_val(tokens).as_static(), "[");
    assert_eq!(next_val(tokens).as_string(), "section");
    assert_eq!(next_val(tokens).as_static(), "]");

    assert_eq!(next_val(tokens).as_string(), "key");
    assert_eq!(next_val(tokens).as_static(), "=");
    assert_eq!(next_val(tokens).as_string(), "\"value\"");
    assert_eq!(next_val(tokens).as_string(), "# A comment");

    assert_eq!(next_val(tokens).as_string(), "1234");
    assert_eq!(next_val(tokens).as_static(), "=");
    assert_eq!(
        next_val(tokens).as_string(),
        "'number-strings-and-literals'"
    );

    assert!(tokens.next().is_none());
}

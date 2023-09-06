use super::*;

const INTEGERS_HEX: &str = r#"
hex1 = 0xDEADBEEF
hex2 = 0xdeadbeef
hex3 = 0xdead_beef
"#;

#[test]
fn hex() {
    let tokens = &mut iter_no_whitespace(INTEGERS_HEX);

    let dead_beef: u64 = 3735928559;

    assert_eq!(next_val(tokens).as_string(), "hex1");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::Equals);
    assert_eq!(next_val(tokens).as_integer(), dead_beef);

    assert_eq!(next_val(tokens).as_string(), "hex2");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::Equals);
    assert_eq!(next_val(tokens).as_integer(), dead_beef);

    assert_eq!(next_val(tokens).as_string(), "hex3");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::Equals);
    assert_eq!(next_val(tokens).as_integer(), dead_beef);

    assert!(tokens.next().is_none());
}

const INTEGERS_OCT: &str = r#"
oct1 = 0o01234567
oct2 = 0o755
"#;

#[test]
fn oct() {
    let tokens = &mut iter_no_whitespace(INTEGERS_OCT);

    assert_eq!(next_val(tokens).as_string(), "oct1");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::Equals);
    assert_eq!(next_val(tokens).as_integer(), 342391);

    assert_eq!(next_val(tokens).as_string(), "oct2");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::Equals);
    assert_eq!(next_val(tokens).as_integer(), 493);

    assert!(tokens.next().is_none());
}

const INTEGERS_BIN: &str = r#"
bin1 = 0b11010110
bin2 = 0b1101_0110
"#;

#[test]
fn bin() {
    let tokens = &mut iter_no_whitespace(INTEGERS_BIN);

    assert_eq!(next_val(tokens).as_string(), "bin1");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::Equals);
    assert_eq!(next_val(tokens).as_integer(), 214);

    assert_eq!(next_val(tokens).as_string(), "bin2");
    assert_eq!(next_val(tokens).as_symbol(), Symbol::Equals);
    assert_eq!(next_val(tokens).as_integer(), 214);

    assert!(tokens.next().is_none());
}

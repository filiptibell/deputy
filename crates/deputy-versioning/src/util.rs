#[must_use]
pub fn is_bare_version(s: &str) -> bool {
    let s = s.trim();
    s.chars().next().is_some_and(|c| c.is_ascii_digit())
}

#[must_use]
pub fn has_version_specifier(s: &str) -> bool {
    let s = s.trim();
    s.chars()
        .next()
        .is_some_and(|c| matches!(c, '^' | '>' | '<' | '=' | '~'))
}

#[must_use]
pub fn trim_version_specifiers(s: &str) -> String {
    let s = s.trim();
    s.trim_start_matches('^')
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('=')
        .trim_start_matches('~')
        .to_string()
}

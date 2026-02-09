#[must_use]
pub fn is_bare_version(s: &str) -> bool {
    let s = s.trim();
    s.chars().next().is_some_and(|c| c.is_ascii_digit())
}

#[must_use]
pub fn has_semver_version_specifier(s: &str) -> bool {
    let s = s.trim();
    s.chars()
        .next()
        .is_some_and(|c| matches!(c, '^' | '>' | '<' | '=' | '~'))
}

#[must_use]
pub fn trim_semver_version_specifiers(s: &str) -> String {
    let s = s.trim();
    s.trim_start_matches('^')
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('=')
        .trim_start_matches('~')
        .to_string()
}

#[must_use]
pub fn has_pep_version_specifier(s: &str) -> bool {
    let s = s.trim();
    s.chars()
        .next()
        .is_some_and(|c| matches!(c, '>' | '<' | '=' | '~' | '!'))
}

#[must_use]
pub fn trim_pep_version_specifiers(s: &str) -> String {
    let s = s.trim();
    s.trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('=')
        .trim_start_matches('~')
        .trim_start_matches('!')
        .trim_start() // pep allows spacing between components...
        .to_string()
}

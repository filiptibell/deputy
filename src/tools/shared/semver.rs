pub fn strip_specifiers(s: &str) -> &str {
    s.trim_start_matches('=')
        .trim_start_matches('>')
        .trim_start_matches(">=")
        .trim_start_matches('<')
        .trim_start_matches("<=")
        .trim_start_matches('~')
        .trim_start_matches('^')
        .trim_start_matches('*')
}

pub fn format_authors(scope_name: &str, authors: &[&str]) -> String {
    if !authors.is_empty() {
        authors
            .iter()
            .map(|&s| strip_author_email(s))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        scope_name.to_string()
    }
}

pub fn strip_author_email(s: &str) -> &str {
    if s.ends_with('>') {
        if let Some(i) = s.find('<') {
            s[..i].trim()
        } else {
            s
        }
    } else {
        s
    }
}

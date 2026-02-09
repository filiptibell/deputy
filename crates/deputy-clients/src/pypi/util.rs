/**
    Normalizes a `PyPI` package name per PEP 503:
    lowercase, with runs of `[-_.]` replaced by a single `-`.
*/
pub fn normalize_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut prev_was_separator = false;

    for ch in name.chars() {
        if ch == '-' || ch == '_' || ch == '.' {
            if !prev_was_separator {
                result.push('-');
                prev_was_separator = true;
            }
        } else {
            result.push(ch.to_ascii_lowercase());
            prev_was_separator = false;
        }
    }

    result
}

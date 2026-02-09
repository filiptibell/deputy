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
    s.trim_start_matches(['^', '>', '<', '=', '~']).to_string()
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
    s.trim_start_matches(['>', '<', '=', '~', '!'])
        .trim_start() // pep allows spacing between operator and version
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_bare_version_digits() {
        assert!(is_bare_version("1.0.0"));
        assert!(is_bare_version("0.1"));
        assert!(is_bare_version("  3.2.1  "));
    }

    #[test]
    fn is_bare_version_with_specifiers() {
        assert!(!is_bare_version("^1.0"));
        assert!(!is_bare_version(">=1.0"));
        assert!(!is_bare_version("~1.0"));
        assert!(!is_bare_version(""));
        assert!(!is_bare_version("  "));
    }

    #[test]
    fn has_semver_specifier_basic() {
        assert!(has_semver_version_specifier("^1.0"));
        assert!(has_semver_version_specifier(">=1.0"));
        assert!(has_semver_version_specifier(">1.0"));
        assert!(has_semver_version_specifier("<1.0"));
        assert!(has_semver_version_specifier("=1.0"));
        assert!(has_semver_version_specifier("~1.0"));
        assert!(!has_semver_version_specifier("1.0"));
        assert!(!has_semver_version_specifier(""));
    }

    #[test]
    fn trim_semver_specifiers() {
        assert_eq!(trim_semver_version_specifiers("^1.0.0"), "1.0.0");
        assert_eq!(trim_semver_version_specifiers(">=1.0.0"), "1.0.0");
        assert_eq!(trim_semver_version_specifiers("~1.0.0"), "1.0.0");
        assert_eq!(trim_semver_version_specifiers("1.0.0"), "1.0.0");
        assert_eq!(trim_semver_version_specifiers("  ^1.0  "), "1.0");
    }

    #[test]
    fn has_pep_specifier_basic() {
        assert!(has_pep_version_specifier(">=1.0"));
        assert!(has_pep_version_specifier(">1.0"));
        assert!(has_pep_version_specifier("<1.0"));
        assert!(has_pep_version_specifier("==1.0"));
        assert!(has_pep_version_specifier("~=1.0"));
        assert!(has_pep_version_specifier("!=1.0"));
        assert!(!has_pep_version_specifier("1.0"));
        assert!(!has_pep_version_specifier(""));
    }

    #[test]
    fn trim_pep_specifiers() {
        assert_eq!(trim_pep_version_specifiers(">=1.0.0"), "1.0.0");
        assert_eq!(trim_pep_version_specifiers("==1.0.0"), "1.0.0");
        assert_eq!(trim_pep_version_specifiers("~=1.0.0"), "1.0.0");
        assert_eq!(trim_pep_version_specifiers("!=1.0.0"), "1.0.0");
        assert_eq!(trim_pep_version_specifiers("===1.0.0"), "1.0.0");
        assert_eq!(trim_pep_version_specifiers("1.0.0"), "1.0.0");
        assert_eq!(trim_pep_version_specifiers("  >= 1.0  "), "1.0");
    }
}

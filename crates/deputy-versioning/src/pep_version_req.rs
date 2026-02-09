use pep440_rs::Operator;

use crate::pep_types::{PepVersion, PepVersionReq};

/**
    Helper trait for deriving versions from a `PepVersionReq`.

    Mostly useful when given a version requirement such as `>=1.2`
    and needing a full, proper version such as `1.2.0` out of it.
*/
pub trait PepVersionReqExt {
    fn minimum_version(&self) -> PepVersion;
}

impl PepVersionReqExt for PepVersionReq {
    fn minimum_version(&self) -> PepVersion {
        possible_versions_for_req(self)
            .into_iter()
            .min()
            .unwrap_or_else(|| "0.0.0".parse().expect("0.0.0 is a valid PEP 440 version"))
    }
}

fn parse_pep_version(s: impl AsRef<str>) -> PepVersion {
    fn parse_pep_version_inner(s: &str) -> PepVersion {
        s.parse().expect("constructed version is valid")
    }
    parse_pep_version_inner(s.as_ref())
}

fn possible_versions_for_req(req: &PepVersionReq) -> Vec<PepVersion> {
    req.inner()
        .iter()
        .flat_map(|spec| {
            let version = spec.version();
            let release = version.release();
            let major = release.first().copied().unwrap_or(0);
            let minor = release.get(1).copied().unwrap_or(0);
            let patch = release.get(2).copied().unwrap_or(0);

            let base_version = parse_pep_version(format!("{major}.{minor}.{patch}"));

            #[allow(clippy::match_same_arms)]
            match spec.operator() {
                // == 1.2.3 or >= 1.2.3 - exact lower bound
                Operator::Equal | Operator::GreaterThanEqual => {
                    vec![base_version]
                }
                // > 1.2.3 - bump the least significant specified component
                Operator::GreaterThan => {
                    if release.len() >= 3 {
                        vec![parse_pep_version(format!("{major}.{minor}.{}", patch + 1))]
                    } else if release.len() >= 2 {
                        vec![parse_pep_version(format!("{major}.{}.0", minor + 1))]
                    } else {
                        vec![parse_pep_version(format!("{}.0.0", major + 1))]
                    }
                }
                // < 1.2.3 - upper bound, return base for candidate pool
                Operator::LessThan => {
                    if release.len() < 3 {
                        if release.len() >= 2 {
                            vec![parse_pep_version(format!("{major}.{minor}.0"))]
                        } else {
                            vec![parse_pep_version(format!("{major}.0.0"))]
                        }
                    } else {
                        vec![base_version]
                    }
                }
                // <= 1.2.3 - upper bound inclusive
                Operator::LessThanEqual => {
                    if release.len() < 3 {
                        if release.len() >= 2 {
                            vec![parse_pep_version(format!("{major}.{}.0", minor + 1))]
                        } else {
                            vec![parse_pep_version(format!("{}.0.0", major + 1))]
                        }
                    } else {
                        vec![base_version]
                    }
                }
                // ~= 1.4.5 means >= 1.4.5, == 1.4.*
                // ~= 1.4 means >= 1.4, == 1.*
                Operator::TildeEqual => {
                    if release.len() >= 3 {
                        vec![
                            base_version,
                            parse_pep_version(format!("{major}.{}.0", minor + 1)),
                        ]
                    } else {
                        vec![
                            parse_pep_version(format!("{major}.{minor}.0")),
                            parse_pep_version(format!("{}.0.0", major + 1)),
                        ]
                    }
                }
                // == 1.2.* - wildcard, produces a range
                Operator::EqualStar => {
                    if release.len() >= 2 {
                        vec![
                            parse_pep_version(format!("{major}.{minor}.0")),
                            parse_pep_version(format!("{major}.{}.0", minor + 1)),
                        ]
                    } else {
                        vec![
                            parse_pep_version(format!("{major}.0.0")),
                            parse_pep_version(format!("{}.0.0", major + 1)),
                        ]
                    }
                }
                // != 1.2.3 or != 1.2.* - exclusion, not useful for minimum bound
                Operator::NotEqual | Operator::NotEqualStar => {
                    vec![base_version]
                }
                // === arbitrary equality
                Operator::ExactEqual => {
                    vec![base_version]
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn min(req: &str) -> String {
        req.parse::<PepVersionReq>()
            .unwrap()
            .minimum_version()
            .to_string()
    }

    // Equal

    #[test]
    fn equal_full() {
        assert_eq!(min("==1.2.3"), "1.2.3");
    }

    #[test]
    fn equal_two_part() {
        assert_eq!(min("==1.2"), "1.2.0");
    }

    // Greater than or equal

    #[test]
    fn gte_full() {
        assert_eq!(min(">=1.2.3"), "1.2.3");
    }

    #[test]
    fn gte_two_part() {
        assert_eq!(min(">=1.2"), "1.2.0");
    }

    // Greater than

    #[test]
    fn gt_full() {
        assert_eq!(min(">1.2.3"), "1.2.4");
    }

    #[test]
    fn gt_two_part() {
        assert_eq!(min(">1.2"), "1.3.0");
    }

    #[test]
    fn gt_one_part() {
        assert_eq!(min(">1"), "2.0.0");
    }

    // Less than

    #[test]
    fn lt_full() {
        assert_eq!(min("<1.2.3"), "1.2.3");
    }

    #[test]
    fn lt_two_part() {
        assert_eq!(min("<1.2"), "1.2.0");
    }

    #[test]
    fn lt_one_part() {
        assert_eq!(min("<2"), "2.0.0");
    }

    // Less than or equal

    #[test]
    fn lte_full() {
        assert_eq!(min("<=1.2.3"), "1.2.3");
    }

    #[test]
    fn lte_two_part() {
        assert_eq!(min("<=1.2"), "1.3.0");
    }

    #[test]
    fn lte_one_part() {
        assert_eq!(min("<=2"), "3.0.0");
    }

    // Tilde equal

    #[test]
    fn tilde_equal_full() {
        // ~=1.4.5 means >=1.4.5, ==1.4.*
        assert_eq!(min("~=1.4.5"), "1.4.5");
    }

    #[test]
    fn tilde_equal_two_part() {
        // ~=1.4 means >=1.4, ==1.*
        assert_eq!(min("~=1.4"), "1.4.0");
    }

    // Equal star (wildcard)

    #[test]
    fn equal_star_minor() {
        // ==1.2.* matches any 1.2.x
        assert_eq!(min("==1.2.*"), "1.2.0");
    }

    #[test]
    fn equal_star_major() {
        // ==1.* matches any 1.x
        assert_eq!(min("==1.*"), "1.0.0");
    }

    // Not equal

    #[test]
    fn not_equal() {
        // !=1.2.3 - exclusion, base version as candidate
        assert_eq!(min("!=1.2.3"), "1.2.3");
    }

    // Not equal star

    #[test]
    fn not_equal_star() {
        assert_eq!(min("!=1.2.*"), "1.2.0");
    }

    // Exact equal

    #[test]
    fn exact_equal() {
        assert_eq!(min("===1.2.3"), "1.2.3");
    }

    // Combined

    #[test]
    fn combined_range() {
        // >=1.2.0, <2.0.0 - minimum should be 1.2.0
        assert_eq!(min(">=1.2.0, <2.0.0"), "1.2.0");
    }

    #[test]
    fn combined_gt_lt() {
        // >0.5, <1.0 - minimum should be 0.6.0
        assert_eq!(min(">0.5, <1.0"), "0.6.0");
    }
}

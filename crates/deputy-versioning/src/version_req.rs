use semver::{Op, Version, VersionReq};

/**
    Helper trait for deriving versions from a `VersionReq`

    Mostly useful when given a version requirement such as `1.0`
    and needing a full, proper version such as `1.0.0` out of it
*/
pub trait VersionReqExt {
    fn minimum_version(&self) -> Version;
}

impl VersionReqExt for VersionReq {
    fn minimum_version(&self) -> Version {
        possible_versions_for_req(self)
            .into_iter()
            .min()
            .unwrap_or_else(|| Version::new(0, 0, 0))
    }
}

fn possible_versions_for_req(req: &VersionReq) -> Vec<Version> {
    req.comparators
        .iter()
        .flat_map(|comp| {
            let base_version =
                Version::new(comp.major, comp.minor.unwrap_or(0), comp.patch.unwrap_or(0));

            match comp.op {
                Op::Exact | Op::GreaterEq => {
                    vec![base_version]
                }
                Op::Greater => {
                    if let Some(minor) = comp.minor {
                        if let Some(patch) = comp.patch {
                            vec![Version::new(comp.major, minor, patch + 1)]
                        } else {
                            vec![Version::new(comp.major, minor + 1, 0)]
                        }
                    } else {
                        vec![Version::new(comp.major + 1, 0, 0)]
                    }
                }
                Op::Less => {
                    if comp.patch.is_none() {
                        if let Some(minor) = comp.minor {
                            vec![Version::new(comp.major, minor, 0)]
                        } else {
                            vec![Version::new(comp.major, 0, 0)]
                        }
                    } else {
                        vec![base_version]
                    }
                }
                Op::LessEq => {
                    if comp.patch.is_none() {
                        if let Some(minor) = comp.minor {
                            vec![Version::new(comp.major, minor + 1, 0)]
                        } else {
                            vec![Version::new(comp.major + 1, 0, 0)]
                        }
                    } else {
                        vec![base_version]
                    }
                }
                Op::Tilde => {
                    if comp.patch.is_some() {
                        vec![
                            base_version,
                            Version::new(comp.major, comp.minor.unwrap() + 1, 0),
                        ]
                    } else if comp.minor.is_some() {
                        // ~I.J is equivalent to =I.J
                        vec![
                            Version::new(comp.major, comp.minor.unwrap(), 0),
                            Version::new(comp.major, comp.minor.unwrap() + 1, 0),
                        ]
                    } else {
                        // ~I is equivalent to =I
                        vec![
                            Version::new(comp.major, 0, 0),
                            Version::new(comp.major + 1, 0, 0),
                        ]
                    }
                }
                Op::Caret => {
                    if comp.major > 0 {
                        vec![base_version, Version::new(comp.major + 1, 0, 0)]
                    } else if let Some(minor) = comp.minor {
                        if minor > 0 {
                            vec![base_version, Version::new(0, minor + 1, 0)]
                        } else if let Some(_patch) = comp.patch {
                            // ^0.0.K is equivalent to =0.0.K
                            vec![base_version]
                        } else {
                            // ^0.0 is equivalent to =0.0
                            vec![Version::new(0, 0, 0)]
                        }
                    } else {
                        // ^0 is equivalent to =0
                        vec![Version::new(0, 0, 0), Version::new(1, 0, 0)]
                    }
                }
                Op::Wildcard => {
                    if let Some(minor) = comp.minor {
                        vec![
                            Version::new(comp.major, minor, 0),
                            Version::new(comp.major, minor + 1, 0),
                        ]
                    } else {
                        vec![
                            Version::new(comp.major, 0, 0),
                            Version::new(comp.major + 1, 0, 0),
                        ]
                    }
                }
                _ => vec![base_version],
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn min(req: &str) -> String {
        req.parse::<VersionReq>()
            .unwrap()
            .minimum_version()
            .to_string()
    }

    // Exact

    #[test]
    fn exact_full() {
        assert_eq!(min("=1.2.3"), "1.2.3");
    }

    #[test]
    fn exact_partial() {
        assert_eq!(min("=1.2"), "1.2.0");
    }

    // Greater than or equal

    #[test]
    fn gte_full() {
        assert_eq!(min(">=1.2.3"), "1.2.3");
    }

    #[test]
    fn gte_partial() {
        assert_eq!(min(">=1.2"), "1.2.0");
    }

    // Greater than

    #[test]
    fn gt_full() {
        assert_eq!(min(">1.2.3"), "1.2.4");
    }

    #[test]
    fn gt_major_minor() {
        assert_eq!(min(">1.2"), "1.3.0");
    }

    #[test]
    fn gt_major_only() {
        assert_eq!(min(">1"), "2.0.0");
    }

    // Less than

    #[test]
    fn lt_full() {
        assert_eq!(min("<1.2.3"), "1.2.3");
    }

    #[test]
    fn lt_major_minor() {
        assert_eq!(min("<1.2"), "1.2.0");
    }

    #[test]
    fn lt_major_only() {
        assert_eq!(min("<2"), "2.0.0");
    }

    // Less than or equal

    #[test]
    fn lte_full() {
        assert_eq!(min("<=1.2.3"), "1.2.3");
    }

    #[test]
    fn lte_major_minor() {
        assert_eq!(min("<=1.2"), "1.3.0");
    }

    #[test]
    fn lte_major_only() {
        assert_eq!(min("<=2"), "3.0.0");
    }

    // Tilde

    #[test]
    fn tilde_full() {
        assert_eq!(min("~1.2.3"), "1.2.3");
    }

    #[test]
    fn tilde_major_minor() {
        assert_eq!(min("~1.2"), "1.2.0");
    }

    #[test]
    fn tilde_major_only() {
        assert_eq!(min("~1"), "1.0.0");
    }

    // Caret

    #[test]
    fn caret_major() {
        assert_eq!(min("^1.2.3"), "1.2.3");
    }

    #[test]
    fn caret_zero_minor() {
        assert_eq!(min("^0.2.3"), "0.2.3");
    }

    #[test]
    fn caret_zero_zero_patch() {
        assert_eq!(min("^0.0.3"), "0.0.3");
    }

    #[test]
    fn caret_zero_zero() {
        assert_eq!(min("^0.0"), "0.0.0");
    }

    #[test]
    fn caret_zero() {
        assert_eq!(min("^0"), "0.0.0");
    }

    // Wildcard

    #[test]
    fn wildcard_major_minor() {
        assert_eq!(min("1.2.*"), "1.2.0");
    }

    #[test]
    fn wildcard_major() {
        assert_eq!(min("1.*"), "1.0.0");
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

    // Fallback

    #[test]
    fn star_only() {
        assert_eq!(min("*"), "0.0.0");
    }
}

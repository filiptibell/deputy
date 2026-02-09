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

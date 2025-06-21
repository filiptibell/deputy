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
                    if comp.patch.is_none() {
                        if comp.minor.is_none() {
                            vec![Version::new(comp.major + 1, 0, 0)]
                        } else {
                            vec![Version::new(comp.major, comp.minor.unwrap() + 1, 0)]
                        }
                    } else {
                        vec![Version::new(
                            comp.major,
                            comp.minor.unwrap(),
                            comp.patch.unwrap() + 1,
                        )]
                    }
                }
                Op::Less => {
                    if comp.patch.is_none() {
                        if comp.minor.is_none() {
                            vec![Version::new(comp.major, 0, 0)]
                        } else {
                            vec![Version::new(comp.major, comp.minor.unwrap(), 0)]
                        }
                    } else {
                        vec![base_version]
                    }
                }
                Op::LessEq => {
                    if comp.patch.is_none() {
                        if comp.minor.is_none() {
                            vec![Version::new(comp.major + 1, 0, 0)]
                        } else {
                            vec![Version::new(comp.major, comp.minor.unwrap() + 1, 0)]
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
                    if comp.minor.is_some() {
                        vec![
                            Version::new(comp.major, comp.minor.unwrap(), 0),
                            Version::new(comp.major, comp.minor.unwrap() + 1, 0),
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

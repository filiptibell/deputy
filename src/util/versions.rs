use semver::{Error, Op, Version, VersionReq};

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

/**
    The latest found version from a comparison.

    Includes metadata about the comparison, the versions, as
    well as the associated data for whatever was compared to.
*/
#[allow(dead_code)]
pub struct LatestVersion<T> {
    pub is_semver_compatible: bool,
    pub is_exactly_compatible: bool,
    pub this_version: Version,
    pub item_version: Version,
    pub item: T,
}

/**
    Helper trait for anything that contains a version string.
*/
pub trait Versioned {
    fn parse_version(&self) -> Result<Version, Error>;

    fn parse_version_req(&self) -> Result<VersionReq, Error> {
        self.parse_version().and_then(|v| v.to_string().parse())
    }

    fn extract_latest_version<I, V>(&self, other_versions: I) -> Option<LatestVersion<V>>
    where
        I: IntoIterator<Item = V>,
        V: Versioned,
    {
        let this_version = self.parse_version().ok()?;
        let this_version_req = self.parse_version_req().ok();

        let mut other_versions = other_versions
            .into_iter()
            .filter_map(|o| match o.parse_version() {
                Ok(v) => Some((o, v)),
                Err(_) => None,
            })
            .filter(|(_, v)| {
                if v.pre.trim().is_empty() {
                    // No prerelease = always consider
                    true
                } else {
                    // Prerelease = only consider if this is also part of the same x.y.z prereleases
                    v.major == this_version.major
                        && v.minor == this_version.minor
                        && v.patch == this_version.patch
                }
            })
            .collect::<Vec<_>>();

        other_versions.sort_by_key(|(_, v)| v.clone());

        other_versions.pop().map(|(item, item_version)| {
            let is_exactly_compatible = item_version
                .to_string()
                .eq_ignore_ascii_case(&this_version.to_string());
            LatestVersion {
                is_semver_compatible: is_exactly_compatible
                    || this_version_req.is_some_and(|req| req.matches(&item_version)),
                is_exactly_compatible,
                this_version,
                item_version,
                item,
            }
        })
    }
}

impl Versioned for Version {
    fn parse_version(&self) -> Result<Version, Error> {
        Ok(self.clone())
    }
}

impl Versioned for String {
    fn parse_version(&self) -> Result<Version, Error> {
        self.parse()
    }
    fn parse_version_req(&self) -> Result<VersionReq, Error> {
        self.parse()
    }
}

impl Versioned for &String {
    fn parse_version(&self) -> Result<Version, Error> {
        self.parse()
    }
    fn parse_version_req(&self) -> Result<VersionReq, Error> {
        self.parse()
    }
}

impl Versioned for &str {
    fn parse_version(&self) -> Result<Version, Error> {
        self.parse()
    }
    fn parse_version_req(&self) -> Result<VersionReq, Error> {
        self.parse()
    }
}

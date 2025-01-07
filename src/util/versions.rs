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

fn trim_version_specifiers(s: String) -> String {
    s.trim_start_matches('^')
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('=')
        .trim_start_matches('~')
        .to_string()
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
    A version to be used for completion purposes.

    Includes the current version, the version that can be completed,
    as well as the associated data for whatever was compared to.

    Note that a completion must not necessarily contain fully valid
    semver versions, since completions can by definition be partial.
*/
#[allow(dead_code)]
pub struct CompletionVersion<T> {
    pub this_version: Option<Version>,
    pub this_version_raw: String,
    pub item_version: Option<Version>,
    pub item_version_raw: String,
    pub item: T,
}

/**
    Helper trait for anything that contains a version string.
*/
pub trait Versioned {
    fn raw_version_string(&self) -> String;

    fn parse_version(&self) -> Result<Version, Error> {
        self.raw_version_string().trim().parse()
    }

    fn parse_version_req(&self) -> Result<VersionReq, Error> {
        self.raw_version_string().trim().parse()
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

    fn extract_completion_versions<I, V>(&self, potential_versions: I) -> Vec<CompletionVersion<V>>
    where
        I: IntoIterator<Item = V>,
        V: Versioned,
    {
        let this_version_raw = match self.parse_version_req() {
            Ok(req) => req.minimum_version().to_string(), // Removes prefixes like '^' in a "correct" manner
            Err(_) => trim_version_specifiers(self.raw_version_string()), // Tries to still remove prefixes, less correct
        };

        let mut potential_versions = potential_versions
            .into_iter()
            .filter_map(|item| {
                let item_version = item.raw_version_string();
                if this_version_raw.is_empty()
                    || (this_version_raw.len() <= item_version.len()
                        && item_version.starts_with(&this_version_raw))
                {
                    Some(item)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        potential_versions.sort_by_key(|item| item.raw_version_string());
        potential_versions.reverse();

        potential_versions
            .into_iter()
            .map(|item| CompletionVersion {
                this_version: this_version_raw.parse().ok(),
                this_version_raw: this_version_raw.clone(),
                item_version: item.parse_version().ok(),
                item_version_raw: item.raw_version_string(),
                item,
            })
            .collect()
    }
}

impl Versioned for Version {
    fn raw_version_string(&self) -> String {
        self.to_string()
    }
}

impl Versioned for String {
    fn raw_version_string(&self) -> String {
        self.to_string()
    }
}

impl Versioned for &String {
    fn raw_version_string(&self) -> String {
        self.to_string()
    }
}

impl Versioned for &str {
    fn raw_version_string(&self) -> String {
        self.to_string()
    }
}

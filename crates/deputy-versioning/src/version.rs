use semver::{Error, Version, VersionReq};

use crate::util::trim_semver_version_specifiers;

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

    /**
       Parses the string into a `Version` object.

       See [`Version::parse`] for more information.
    */
    #[allow(clippy::missing_errors_doc)]
    fn parse_version(&self) -> Result<Version, Error> {
        self.raw_version_string().trim().parse()
    }

    /**
       Parses the string into a `VersionReq` object.

       See [`VersionReq::parse`] for more information.
    */
    #[allow(clippy::missing_errors_doc)]
    fn parse_version_req(&self) -> Result<VersionReq, Error> {
        self.raw_version_string().trim().parse()
    }

    fn deprecated(&self) -> bool {
        false
    }

    fn extract_latest_version_filtered<I, V, F>(
        &self,
        other_versions: I,
        filter_fn: F,
    ) -> Option<LatestVersion<V>>
    where
        I: IntoIterator<Item = V>,
        V: Versioned,
        F: Fn(&LatestVersion<V>) -> bool,
    {
        let this_version = self.parse_version().ok()?;
        let this_version_req = self.parse_version_req().ok();

        let other_versions = other_versions
            .into_iter()
            .filter(|v| !v.deprecated())
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

        let mut latest_versions = other_versions
            .into_iter()
            .map(|(item, item_version)| {
                let is_exactly_compatible = item_version
                    .to_string()
                    .eq_ignore_ascii_case(&this_version.to_string());
                LatestVersion {
                    is_semver_compatible: is_exactly_compatible
                        || this_version_req
                            .as_ref()
                            .is_some_and(|req| req.matches(&item_version)),
                    is_exactly_compatible,
                    this_version: this_version.clone(),
                    item_version,
                    item,
                }
            })
            .collect::<Vec<_>>();

        latest_versions.retain(|latest_version| filter_fn(latest_version));
        latest_versions.sort_by_key(|latest_version| latest_version.item_version.clone());
        latest_versions.pop()
    }

    fn extract_latest_version<I, V>(&self, other_versions: I) -> Option<LatestVersion<V>>
    where
        I: IntoIterator<Item = V>,
        V: Versioned,
    {
        self.extract_latest_version_filtered(other_versions, |_| true)
    }

    fn extract_completion_versions_filtered<I, V, F>(
        &self,
        potential_versions: I,
        filter_fn: F,
    ) -> Vec<CompletionVersion<V>>
    where
        I: IntoIterator<Item = V>,
        V: Versioned,
        F: Fn(&V) -> bool,
    {
        // Try to remove prefixes from partial string - this is not necessarily
        // 100% correct but unfortunately parsing as semver is not always possible
        let this_version_raw = trim_semver_version_specifiers(&self.raw_version_string());

        let mut potential_versions = potential_versions
            .into_iter()
            .filter(|v| !v.deprecated())
            .filter_map(|item| {
                if this_version_raw.is_empty() {
                    return Some(item);
                }

                let item_version = item.raw_version_string();
                if this_version_raw.len() <= item_version.len()
                    && item_version.starts_with(&this_version_raw)
                {
                    Some(item)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        potential_versions.retain(|v| filter_fn(v));

        potential_versions.sort_unstable_by(|a, b| {
            if let Ok(v_a) = a.parse_version()
                && let Ok(v_b) = b.parse_version()
            {
                return v_a.cmp(&v_b);
            }
            let s_a = a.raw_version_string();
            let s_b = b.raw_version_string();
            s_a.cmp(&s_b)
        });

        potential_versions.reverse(); // Latest versions first

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

    fn extract_completion_versions<I, V>(&self, potential_versions: I) -> Vec<CompletionVersion<V>>
    where
        I: IntoIterator<Item = V>,
        V: Versioned,
    {
        self.extract_completion_versions_filtered(potential_versions, |_| true)
    }
}

impl Versioned for Version {
    fn raw_version_string(&self) -> String {
        self.to_string()
    }
}

impl Versioned for String {
    fn raw_version_string(&self) -> String {
        self.clone()
    }
}

impl Versioned for &String {
    fn raw_version_string(&self) -> String {
        (*self).clone()
    }
}

impl Versioned for &str {
    fn raw_version_string(&self) -> String {
        (*self).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper that wraps a version string with an optional deprecated flag
    struct TestVersion {
        version: String,
        deprecated: bool,
    }

    impl TestVersion {
        fn new(version: &str) -> Self {
            Self {
                version: version.to_string(),
                deprecated: false,
            }
        }

        fn deprecated(version: &str) -> Self {
            Self {
                version: version.to_string(),
                deprecated: true,
            }
        }
    }

    impl Versioned for TestVersion {
        fn raw_version_string(&self) -> String {
            self.version.clone()
        }

        fn deprecated(&self) -> bool {
            self.deprecated
        }
    }

    // extract_latest_version

    #[test]
    fn latest_version_picks_highest() {
        let versions = vec![
            TestVersion::new("1.0.0"),
            TestVersion::new("1.1.0"),
            TestVersion::new("1.2.0"),
        ];
        let result = "1.0.0".extract_latest_version(versions).unwrap();
        assert_eq!(result.item_version.to_string(), "1.2.0");
    }

    #[test]
    fn latest_version_exact_match() {
        let versions = vec![
            TestVersion::new("1.0.0"),
            TestVersion::new("1.1.0"),
            TestVersion::new("1.2.0"),
        ];
        let result = "1.2.0".extract_latest_version(versions).unwrap();
        assert!(result.is_exactly_compatible);
        assert!(result.is_semver_compatible);
    }

    #[test]
    fn latest_version_incompatible_when_newer_exists() {
        // Input "1.0.0" parses as req "^1.0.0", which matches 1.x but not 2.x
        let versions = vec![
            TestVersion::new("1.0.0"),
            TestVersion::new("1.5.0"),
            TestVersion::new("2.0.0"),
        ];
        let result = "1.0.0".extract_latest_version(versions).unwrap();
        assert_eq!(result.item_version.to_string(), "2.0.0");
        // 2.0.0 is the latest overall, but not semver-compatible with ^1.0.0
        assert!(!result.is_semver_compatible);
    }

    #[test]
    fn latest_version_skips_deprecated() {
        let versions = vec![
            TestVersion::new("1.0.0"),
            TestVersion::deprecated("1.5.0"),
            TestVersion::new("1.2.0"),
        ];
        let result = "1.0.0".extract_latest_version(versions).unwrap();
        // 1.5.0 is deprecated, so latest should be 1.2.0
        assert_eq!(result.item_version.to_string(), "1.2.0");
    }

    #[test]
    fn latest_version_filters_prerelease() {
        let versions = vec![
            TestVersion::new("1.0.0"),
            TestVersion::new("1.1.0"),
            TestVersion::new("2.0.0-alpha.1"),
        ];
        // Prerelease for 2.0.0 should be excluded since our version is 1.0.0
        let result = "1.0.0".extract_latest_version(versions).unwrap();
        assert_eq!(result.item_version.to_string(), "1.1.0");
    }

    #[test]
    fn latest_version_includes_same_base_prerelease() {
        let versions = vec![
            TestVersion::new("1.0.0"),
            TestVersion::new("1.0.0-beta.2"),
            TestVersion::new("1.0.0-beta.1"),
        ];
        // Prereleases for 1.0.0 should be included since our version is also 1.0.0
        let result = "1.0.0".extract_latest_version(versions).unwrap();
        assert_eq!(result.item_version.to_string(), "1.0.0");
    }

    #[test]
    fn latest_version_with_filter() {
        let versions = vec![
            TestVersion::new("1.0.0"),
            TestVersion::new("1.1.0"),
            TestVersion::new("1.2.0"),
        ];
        // Filter to only versions where minor < 2
        let result = "1.0.0"
            .extract_latest_version_filtered(versions, |v| v.item_version.minor < 2)
            .unwrap();
        assert_eq!(result.item_version.to_string(), "1.1.0");
    }

    #[test]
    fn latest_version_none_when_no_match() {
        let versions: Vec<TestVersion> = vec![];
        let result = "1.0.0".extract_latest_version(versions);
        assert!(result.is_none());
    }

    #[test]
    fn latest_version_none_when_invalid_input() {
        let versions = vec![TestVersion::new("1.0.0")];
        // "not-a-version" can't be parsed, so extract_latest_version returns None
        let result = "not-a-version".extract_latest_version(versions);
        assert!(result.is_none());
    }

    // extract_completion_versions

    #[test]
    fn completion_versions_prefix_filter() {
        let versions = vec![
            TestVersion::new("1.0.0"),
            TestVersion::new("1.1.0"),
            TestVersion::new("2.0.0"),
        ];
        let results = "1.".extract_completion_versions(versions);
        assert_eq!(results.len(), 2);
        // Latest first
        assert_eq!(results[0].item_version_raw, "1.1.0");
        assert_eq!(results[1].item_version_raw, "1.0.0");
    }

    #[test]
    fn completion_versions_empty_prefix() {
        let versions = vec![TestVersion::new("1.0.0"), TestVersion::new("2.0.0")];
        let results = "".extract_completion_versions(versions);
        // Empty prefix should return all versions
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn completion_versions_strips_specifier() {
        let versions = vec![
            TestVersion::new("1.0.0"),
            TestVersion::new("1.1.0"),
            TestVersion::new("2.0.0"),
        ];
        // "^1." should strip "^" and filter by "1."
        let results = "^1.".extract_completion_versions(versions);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn completion_versions_skips_deprecated() {
        let versions = vec![
            TestVersion::new("1.0.0"),
            TestVersion::deprecated("1.1.0"),
            TestVersion::new("1.2.0"),
        ];
        let results = "1.".extract_completion_versions(versions);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].item_version_raw, "1.2.0");
        assert_eq!(results[1].item_version_raw, "1.0.0");
    }

    #[test]
    fn completion_versions_sorted_latest_first() {
        let versions = vec![
            TestVersion::new("1.2.0"),
            TestVersion::new("1.0.0"),
            TestVersion::new("1.10.0"),
            TestVersion::new("1.1.0"),
        ];
        let results = "1.".extract_completion_versions(versions);
        let raw: Vec<_> = results
            .iter()
            .map(|r| r.item_version_raw.as_str())
            .collect();
        assert_eq!(raw, vec!["1.10.0", "1.2.0", "1.1.0", "1.0.0"]);
    }

    #[test]
    fn completion_versions_no_match() {
        let versions = vec![TestVersion::new("1.0.0"), TestVersion::new("1.1.0")];
        let results = "2.".extract_completion_versions(versions);
        assert!(results.is_empty());
    }
}

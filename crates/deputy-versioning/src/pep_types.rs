use std::fmt;
use std::str::FromStr;

use pep440_rs::{
    Version as Pep440Version, VersionParseError, VersionSpecifiers, VersionSpecifiersParseError,
};

/**
    Newtype wrapper around a PEP 440 version.

    Provides a similar interface to `semver::Version`
    with named accessors for major, minor, and patch.
*/
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PepVersion(Pep440Version);

impl PepVersion {
    #[must_use]
    pub fn major(&self) -> u64 {
        self.0.release().first().copied().unwrap_or(0)
    }

    #[must_use]
    pub fn minor(&self) -> u64 {
        self.0.release().get(1).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn patch(&self) -> u64 {
        self.0.release().get(2).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn is_prerelease(&self) -> bool {
        self.0.any_prerelease()
    }

    #[must_use]
    pub fn is_stable(&self) -> bool {
        self.0.is_stable()
    }

    #[must_use]
    pub fn inner(&self) -> &Pep440Version {
        &self.0
    }
}

impl From<Pep440Version> for PepVersion {
    fn from(v: Pep440Version) -> Self {
        Self(v)
    }
}

impl fmt::Display for PepVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for PepVersion {
    type Err = VersionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Pep440Version::from_str(s).map(Self)
    }
}

/**
    Newtype wrapper around PEP 440 version specifiers.

    Provides a similar interface to `semver::VersionReq`
    with a `matches` method for checking version compatibility.
*/
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PepVersionReq(VersionSpecifiers);

impl PepVersionReq {
    #[must_use]
    pub fn matches(&self, version: &PepVersion) -> bool {
        self.0.contains(version.inner())
    }

    #[must_use]
    pub fn inner(&self) -> &VersionSpecifiers {
        &self.0
    }
}

impl From<VersionSpecifiers> for PepVersionReq {
    fn from(v: VersionSpecifiers) -> Self {
        Self(v)
    }
}

impl fmt::Display for PepVersionReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for PepVersionReq {
    type Err = VersionSpecifiersParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        VersionSpecifiers::from_str(s).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> PepVersion {
        s.parse().unwrap()
    }

    fn req(s: &str) -> PepVersionReq {
        s.parse().unwrap()
    }

    // PepVersion accessors

    #[test]
    fn accessors_three_part() {
        let ver = v("1.2.3");
        assert_eq!(ver.major(), 1);
        assert_eq!(ver.minor(), 2);
        assert_eq!(ver.patch(), 3);
    }

    #[test]
    fn accessors_two_part() {
        let ver = v("1.2");
        assert_eq!(ver.major(), 1);
        assert_eq!(ver.minor(), 2);
        assert_eq!(ver.patch(), 0);
    }

    #[test]
    fn accessors_one_part() {
        let ver = v("5");
        assert_eq!(ver.major(), 5);
        assert_eq!(ver.minor(), 0);
        assert_eq!(ver.patch(), 0);
    }

    #[test]
    fn accessors_four_part() {
        // PEP 440 allows arbitrary release segments
        let ver = v("1.2.3.4");
        assert_eq!(ver.major(), 1);
        assert_eq!(ver.minor(), 2);
        assert_eq!(ver.patch(), 3);
    }

    // Prerelease / stable

    #[test]
    fn stable_version() {
        let ver = v("1.0.0");
        assert!(!ver.is_prerelease());
        assert!(ver.is_stable());
    }

    #[test]
    fn alpha_prerelease() {
        let ver = v("1.0.0a1");
        assert!(ver.is_prerelease());
        assert!(!ver.is_stable());
    }

    #[test]
    fn beta_prerelease() {
        let ver = v("1.0.0b2");
        assert!(ver.is_prerelease());
        assert!(!ver.is_stable());
    }

    #[test]
    fn release_candidate() {
        let ver = v("1.0.0rc1");
        assert!(ver.is_prerelease());
        assert!(!ver.is_stable());
    }

    #[test]
    fn dev_release() {
        let ver = v("1.0.0.dev1");
        assert!(ver.is_prerelease());
        assert!(!ver.is_stable());
    }

    #[test]
    fn post_release_is_stable() {
        let ver = v("1.0.0.post1");
        assert!(!ver.is_prerelease());
        assert!(ver.is_stable());
    }

    // Ordering

    #[test]
    fn ordering_basic() {
        assert!(v("1.0.0") < v("1.1.0"));
        assert!(v("1.1.0") < v("2.0.0"));
        assert!(v("1.0.0") < v("1.0.1"));
    }

    #[test]
    fn ordering_prerelease_before_release() {
        assert!(v("1.0.0a1") < v("1.0.0"));
        assert!(v("1.0.0b1") < v("1.0.0"));
        assert!(v("1.0.0rc1") < v("1.0.0"));
    }

    // Display / FromStr round-trip

    #[test]
    fn display_roundtrip() {
        let ver = v("1.2.3");
        let s = ver.to_string();
        let ver2: PepVersion = s.parse().unwrap();
        assert_eq!(ver, ver2);
    }

    // PepVersionReq matching

    #[test]
    fn req_matches_equal() {
        let r = req("==1.2.3");
        assert!(r.matches(&v("1.2.3")));
        assert!(!r.matches(&v("1.2.4")));
    }

    #[test]
    fn req_matches_gte() {
        let r = req(">=1.2.0");
        assert!(r.matches(&v("1.2.0")));
        assert!(r.matches(&v("1.3.0")));
        assert!(!r.matches(&v("1.1.0")));
    }

    #[test]
    fn req_matches_combined() {
        let r = req(">=1.0, <2.0");
        assert!(r.matches(&v("1.0.0")));
        assert!(r.matches(&v("1.9.9")));
        assert!(!r.matches(&v("2.0.0")));
        assert!(!r.matches(&v("0.9.0")));
    }

    #[test]
    fn req_matches_wildcard() {
        let r = req("==1.2.*");
        assert!(r.matches(&v("1.2.0")));
        assert!(r.matches(&v("1.2.99")));
        assert!(!r.matches(&v("1.3.0")));
    }

    #[test]
    fn req_matches_tilde_equal() {
        // ~=1.4.5 means >=1.4.5, ==1.4.*
        let r = req("~=1.4.5");
        assert!(r.matches(&v("1.4.5")));
        assert!(r.matches(&v("1.4.9")));
        assert!(!r.matches(&v("1.5.0")));
        assert!(!r.matches(&v("1.4.4")));
    }
}

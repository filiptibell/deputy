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

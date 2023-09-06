#![allow(clippy::enum_variant_names)]

use semver::{Version, VersionReq};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum DependencySpecError {
    #[error("dependency name contains invalid character '{0}'")]
    InvalidName(char),
    #[error("dependency version contains invalid character '{0}'")]
    InvalidVersion(char),
    #[error("dependency version contains invalid semver - {0}")]
    InvalidSemver(String),
}

#[derive(Debug, Clone)]
pub struct DependencySpec {
    pub name: String,
    pub version: Option<Version>,
    pub version_req: VersionReq,
}

impl DependencySpec {
    pub fn parse(
        name: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Result<Self, DependencySpecError> {
        let version = version.as_ref();

        let name = validate_name(name.as_ref())?;
        let version_req = validate_version(version)?;

        Ok(Self {
            name,
            version: Version::parse(version).ok(),
            version_req,
        })
    }
}

fn is_valid_naming_char(char: char) -> bool {
    char == '-' || char == '_' || char.is_ascii_alphanumeric()
}

fn is_valid_version_char(char: char) -> bool {
    char == '.' || char == '-' || char == '_' || char.is_ascii_alphanumeric()
}

fn validate_name(name: impl AsRef<str>) -> Result<String, DependencySpecError> {
    let name = name.as_ref();
    if let Some(invalid_char) = name.chars().find(|c| !is_valid_naming_char(*c)) {
        Err(DependencySpecError::InvalidName(invalid_char))
    } else {
        Ok(name.to_string())
    }
}

fn validate_version(version: impl AsRef<str>) -> Result<VersionReq, DependencySpecError> {
    let mut version = version.as_ref();
    if version.starts_with('v') {
        version = &version[1..]
    }
    if let Some(invalid_char) = version.chars().find(|c| !is_valid_version_char(*c)) {
        Err(DependencySpecError::InvalidVersion(invalid_char))
    } else {
        match VersionReq::parse(version) {
            Err(semver_err) => Err(DependencySpecError::InvalidSemver(semver_err.to_string())),
            Ok(semver) => Ok(semver),
        }
    }
}

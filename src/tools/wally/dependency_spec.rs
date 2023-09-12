use std::str::FromStr;

use semver::{Version, VersionReq};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum DependencySpecError {
    #[error("missing package author")]
    MissingAuthor,
    #[error("missing package name")]
    MissingName,
    #[error("missing package version")]
    MissingVersion,
    #[error("package author contains invalid character '{0}'")]
    InvalidAuthor(char),
    #[error("package name contains invalid character '{0}'")]
    InvalidName(char),
    #[error("package version contains invalid character '{0}'")]
    InvalidVersion(char),
    #[error("package version contains invalid semver - {0}")]
    InvalidSemver(String),
}

#[derive(Debug, Clone)]
pub struct DependencySpec {
    pub author: String,
    pub name: String,
    pub version: Option<Version>,
    pub version_req: VersionReq,
    pub version_text: String,
}

impl FromStr for DependencySpec {
    type Err = DependencySpecError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(DependencySpecError::MissingAuthor);
        }

        let idx_slash = s.char_indices().find(|(_, c)| c == &'/');
        if idx_slash.is_none() {
            validate_author(s)?;
            return Err(DependencySpecError::MissingName);
        }

        let idx_slash = idx_slash.unwrap().0;
        if idx_slash == s.len() - 1 {
            return Err(DependencySpecError::MissingName);
        }

        let package_author = validate_author(&s[..idx_slash])?;

        let idx_at = s.char_indices().find(|(_, c)| c == &'@');
        if idx_at.is_none() {
            if idx_slash > s.len() - 1 {
                validate_name(&s[idx_slash + 1..])?;
            }
            return Err(DependencySpecError::MissingVersion);
        }

        let idx_at = idx_at.unwrap().0;
        let package_name = validate_name(&s[idx_slash + 1..idx_at])?;

        if idx_at == s.len() - 1 {
            return Err(DependencySpecError::MissingVersion);
        }

        let version_text = &s[idx_at + 1..];
        let package_version = validate_version(version_text)?;

        Ok(Self {
            author: package_author,
            name: package_name,
            version: Version::parse(version_text).ok(),
            version_req: package_version,
            version_text: version_text.to_string(),
        })
    }
}

fn is_valid_naming_char(char: char) -> bool {
    char == '-' || char == '_' || char.is_ascii_alphanumeric()
}

fn is_valid_version_char(char: char) -> bool {
    char == '.' || char == '-' || char == '_' || char.is_ascii_alphanumeric()
}

fn validate_author(author: impl AsRef<str>) -> Result<String, DependencySpecError> {
    let author = author.as_ref();
    if let Some(invalid_char) = author.chars().find(|c| !is_valid_naming_char(*c)) {
        Err(DependencySpecError::InvalidAuthor(invalid_char))
    } else {
        Ok(author.to_string())
    }
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

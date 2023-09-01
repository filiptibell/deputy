use std::{fmt, str::FromStr};

use semver::Version;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ToolSpecError {
    #[error("missing tool author")]
    MissingAuthor,
    #[error("missing tool name")]
    MissingName,
    #[error("missing tool version")]
    MissingVersion,
    #[error("tool author contains invalid character '{0}'")]
    InvalidAuthor(char),
    #[error("tool name contains invalid character '{0}'")]
    InvalidName(char),
    #[error("tool version contains invalid character '{0}'")]
    InvalidVersion(char),
    #[error("tool version contains invalid semver - {0}")]
    InvalidSemver(String),
}

#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub author: String,
    pub name: String,
    pub version: Version,
}

impl fmt::Display for ToolSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}@{}", self.author, self.name, self.version)
    }
}

impl FromStr for ToolSpec {
    type Err = ToolSpecError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ToolSpecError::MissingAuthor);
        }

        let idx_slash = s.char_indices().find(|(_, c)| c == &'/');
        if idx_slash.is_none() {
            validate_author(s)?;
            return Err(ToolSpecError::MissingName);
        }

        let idx_slash = idx_slash.unwrap().0;
        if idx_slash == s.len() - 1 {
            return Err(ToolSpecError::MissingName);
        }

        let tool_author = validate_author(&s[..idx_slash])?;

        let idx_at = s.char_indices().find(|(_, c)| c == &'@');
        if idx_at.is_none() {
            if idx_slash > s.len() - 1 {
                validate_name(&s[idx_slash + 1..])?;
            }
            return Err(ToolSpecError::MissingVersion);
        }

        let idx_at = idx_at.unwrap().0;
        let tool_name = validate_name(&s[idx_slash + 1..idx_at])?;

        if idx_at == s.len() - 1 {
            return Err(ToolSpecError::MissingVersion);
        }

        let tool_version = validate_version(&s[idx_at + 1..])?;

        Ok(Self {
            author: tool_author,
            name: tool_name,
            version: tool_version,
        })
    }
}

fn is_valid_naming_char(char: char) -> bool {
    char == '-' || char == '_' || char.is_ascii_alphanumeric()
}

fn is_valid_version_char(char: char) -> bool {
    char == '.' || char == '-' || char == '_' || char.is_ascii_alphanumeric()
}

fn validate_author(author: impl AsRef<str>) -> Result<String, ToolSpecError> {
    let author = author.as_ref();
    if let Some(invalid_char) = author.chars().find(|c| !is_valid_naming_char(*c)) {
        Err(ToolSpecError::InvalidAuthor(invalid_char))
    } else {
        Ok(author.to_string())
    }
}

fn validate_name(name: impl AsRef<str>) -> Result<String, ToolSpecError> {
    let name = name.as_ref();
    if let Some(invalid_char) = name.chars().find(|c| !is_valid_naming_char(*c)) {
        Err(ToolSpecError::InvalidName(invalid_char))
    } else {
        Ok(name.to_string())
    }
}

fn validate_version(version: impl AsRef<str>) -> Result<Version, ToolSpecError> {
    let mut version = version.as_ref();
    if version.starts_with('v') {
        version = &version[1..]
    }
    if let Some(invalid_char) = version.chars().find(|c| !is_valid_version_char(*c)) {
        Err(ToolSpecError::InvalidVersion(invalid_char))
    } else {
        match Version::parse(version) {
            Err(semver_err) => Err(ToolSpecError::InvalidSemver(semver_err.to_string())),
            Ok(semver) => Ok(semver),
        }
    }
}

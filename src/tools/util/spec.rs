#![allow(dead_code)]

use std::ops::Range;

use semver::VersionReq;
use thiserror::Error;
use tower_lsp::lsp_types::DiagnosticSeverity;

use crate::lang::json::*;
use crate::lang::toml::*;
use crate::lang::*;

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("internal error - missing extras in struct")]
    InternalMissingExtras,
    #[error("internal error - invalid extras in struct")]
    InternalInvalidExtras,
    #[error("missing author")]
    MissingAuthor,
    #[error("missing name")]
    MissingName,
    #[error("missing release tag")]
    MissingTag,
    #[error("missing version")]
    MissingVersion,
    #[error("author contains invalid character '{0}'")]
    InvalidAuthor(char),
    #[error("name contains invalid character '{0}'")]
    InvalidName(char),
    #[error("release tag contains invalid character '{0}'")]
    InvalidTag(char),
    #[error("version contains invalid character '{0}'")]
    InvalidVersion(char),
    #[error("version contains invalid semver - {0}")]
    InvalidSemver(#[from] semver::Error),
}

impl SpecError {
    pub const fn diagnostic_severity(&self) -> DiagnosticSeverity {
        // Missing author / name / version usually happens
        // when the user is typing, and is not really an error
        if matches!(
            self,
            Self::MissingAuthor | Self::MissingName | Self::MissingTag | Self::MissingVersion
        ) {
            DiagnosticSeverity::WARNING
        } else {
            DiagnosticSeverity::ERROR
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpecAftman {
    pub author: String,
    pub name: String,
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct SpecCargo {
    pub name: String,
    pub version: String,
    pub version_req: VersionReq,
}

#[derive(Debug, Clone)]
pub struct SpecForeman {
    pub author: String,
    pub name: String,
    pub version: String,
    pub version_req: VersionReq,
}

#[derive(Debug, Clone)]
pub struct SpecJavaScript {
    pub name: String,
    pub version: String,
    pub version_req: VersionReq,
}

#[derive(Debug, Clone)]
pub struct SpecWally {
    pub author: String,
    pub name: String,
    pub version: String,
    pub version_req: VersionReq,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spec<K: LangString, V: LangString, E: LangValue> {
    key: K,
    value: V,
    extras: Option<E>,
}

impl<K: LangString, V: LangString, E: LangValue> Spec<K, V, E> {
    pub fn from_key_value_pair(key: &K, value: &V) -> Self {
        Self {
            key: key.clone(),
            value: value.clone(),
            extras: None,
        }
    }

    pub fn from_key_value_pair_with_extras(key: &K, value: &V, extras: &E) -> Self {
        Self {
            key: key.clone(),
            value: value.clone(),
            extras: Some(extras.clone()),
        }
    }

    pub fn full_span(&self) -> Range<usize> {
        let start = self.key.span();
        let end = self.value.span();
        Range {
            start: start.start,
            end: end.end,
        }
    }

    pub fn key_span(&self) -> Range<usize> {
        self.key.span()
    }

    pub fn key_text(&self) -> &str {
        self.key.value()
    }

    pub fn key_source(&self) -> &str {
        self.key.source()
    }

    pub fn value_span(&self) -> Range<usize> {
        self.value.span()
    }

    pub fn value_text(&self) -> &str {
        self.value.value()
    }

    pub fn value_source(&self) -> &str {
        self.value.source()
    }

    pub fn extras_span(&self) -> Option<Range<usize>> {
        self.extras.as_ref().map(|e| e.span())
    }

    pub fn extras_value(&self) -> Option<&E> {
        self.extras.as_ref()
    }

    pub fn extras_source(&self) -> Option<&str> {
        self.extras.as_ref().map(|e| e.source())
    }
}

impl Spec<TomlString, TomlString, TomlValue> {
    pub fn as_aftman(&self) -> Result<SpecAftman, SpecError> {
        let v = self.value_text();
        if v.is_empty() {
            return Err(SpecError::MissingAuthor);
        }

        let idx_last = v.char_indices().last().map(|(i, _)| i).unwrap();
        let idx_slash = find_slash(v).ok_or(SpecError::MissingName)?;
        let idx_at = find_at_sign(v, idx_slash).ok_or(SpecError::MissingTag)?;

        if idx_slash == idx_last {
            return Err(SpecError::MissingName);
        } else if idx_at == idx_last {
            return Err(SpecError::MissingTag);
        }

        Ok(SpecAftman {
            author: validate_author(&v[..idx_slash])?,
            name: validate_name(&v[idx_slash + 1..idx_at])?,
            tag: validate_tag(&v[idx_at + 1..])?,
        })
    }

    pub fn as_cargo(&self) -> Result<SpecCargo, SpecError> {
        let k = self.key_text();
        if k.is_empty() {
            return Err(SpecError::MissingAuthor);
        }

        let v = self.value_text();
        if v.is_empty() {
            return Err(SpecError::MissingVersion);
        }

        Ok(SpecCargo {
            name: validate_name(k)?,
            version_req: validate_version(v)?,
            version: v.to_string(),
        })
    }

    pub fn as_foreman(&self) -> Result<SpecForeman, SpecError> {
        let v = self.value_text();
        if v.is_empty() {
            return Err(SpecError::MissingAuthor);
        }

        let e = match self.extras_value() {
            None => return Err(SpecError::InternalMissingExtras),
            Some(e) => e,
        };
        let e = match e.as_string() {
            None => return Err(SpecError::InternalInvalidExtras),
            Some(e) => e.value(),
        };
        if e.is_empty() {
            return Err(SpecError::MissingVersion);
        }

        let idx_last = v.char_indices().last().map(|(i, _)| i).unwrap();
        let idx_slash = find_slash(v).ok_or(SpecError::MissingName)?;
        if idx_slash == idx_last {
            return Err(SpecError::MissingName);
        }

        Ok(SpecForeman {
            author: validate_author(&v[..idx_slash])?,
            name: validate_name(&v[idx_slash + 1..])?,
            version_req: validate_version(e)?,
            version: e.to_string(),
        })
    }

    pub fn as_wally(&self) -> Result<SpecWally, SpecError> {
        let v = self.value_text();
        if v.is_empty() {
            return Err(SpecError::MissingAuthor);
        }

        let idx_last = v.char_indices().last().map(|(i, _)| i).unwrap();
        let idx_slash = find_slash(v).ok_or(SpecError::MissingName)?;
        let idx_at = find_at_sign(v, idx_slash).ok_or(SpecError::MissingVersion)?;

        if idx_slash == idx_last {
            return Err(SpecError::MissingName);
        } else if idx_at == idx_last {
            return Err(SpecError::MissingVersion);
        }

        let version = &v[idx_at + 1..];

        Ok(SpecWally {
            author: validate_author(&v[..idx_slash])?,
            name: validate_name(&v[idx_slash + 1..idx_at])?,
            version_req: validate_version(version)?,
            version: version.to_string(),
        })
    }
}

impl Spec<JsonString, JsonString, JsonValue> {
    pub fn as_javascript(&self) -> Result<SpecJavaScript, SpecError> {
        let k = self.key_text();
        if k.is_empty() {
            return Err(SpecError::MissingAuthor);
        }

        let v = self.value_text();
        if v.is_empty() {
            return Err(SpecError::MissingVersion);
        }

        Ok(SpecJavaScript {
            name: validate_name_javascript(k)?,
            version_req: validate_version(v)?,
            version: v.to_string(),
        })
    }
}

fn find_slash(s: &str) -> Option<usize> {
    s.char_indices().find(|(_, c)| c == &'/').map(|(i, _)| i)
}

fn find_at_sign(s: &str, after: usize) -> Option<usize> {
    s.char_indices()
        .find(|(i, c)| *i >= after && c == &'@')
        .map(|(i, _)| i)
}

fn is_valid_naming_char(char: char) -> bool {
    char == '-' || char == '_' || char.is_ascii_alphanumeric()
}

fn is_valid_naming_char_javascript(char: char) -> bool {
    char == '-' || char == '_' || char == '@' || char == '/' || char.is_ascii_alphanumeric()
}

fn is_valid_version_char(char: char) -> bool {
    char == '.' || char == '-' || char == '_' || char.is_ascii_alphanumeric()
}

fn is_valid_version_prefix_char(char: char) -> bool {
    char == '=' || char == '^' || char == '~' || char == '>' || char == '<'
}

fn validate_author(author: impl AsRef<str>) -> Result<String, SpecError> {
    let author = author.as_ref();
    if let Some(invalid_char) = author.chars().find(|c| !is_valid_naming_char(*c)) {
        Err(SpecError::InvalidAuthor(invalid_char))
    } else {
        Ok(author.to_string())
    }
}

fn validate_name(name: impl AsRef<str>) -> Result<String, SpecError> {
    let name = name.as_ref();
    if let Some(invalid_char) = name.chars().find(|c| !is_valid_naming_char(*c)) {
        Err(SpecError::InvalidName(invalid_char))
    } else {
        Ok(name.to_string())
    }
}

fn validate_name_javascript(name: impl AsRef<str>) -> Result<String, SpecError> {
    let name = name.as_ref();
    if let Some(invalid_char) = name.chars().find(|c| !is_valid_naming_char_javascript(*c)) {
        Err(SpecError::InvalidName(invalid_char))
    } else {
        Ok(name.to_string())
    }
}

fn validate_tag(version: impl AsRef<str>) -> Result<String, SpecError> {
    let mut version = version.as_ref();
    if version.starts_with('v') {
        version = &version[1..]
    }
    if let Some(invalid_char) = version.chars().find(|c| !is_valid_version_char(*c)) {
        Err(SpecError::InvalidTag(invalid_char))
    } else {
        Ok(version.to_string())
    }
}

fn validate_version(version: impl AsRef<str>) -> Result<VersionReq, SpecError> {
    let version = version.as_ref();
    if let Some(invalid_char) = version
        .chars()
        .find(|c| !is_valid_version_char(*c) && !is_valid_version_prefix_char(*c))
    {
        Err(SpecError::InvalidVersion(invalid_char))
    } else {
        match VersionReq::parse(version) {
            Err(semver_err) => Err(SpecError::InvalidSemver(semver_err)),
            Ok(semver) => Ok(semver),
        }
    }
}

use std::fmt;

pub type CratesResult<T, E = CratesError> = Result<T, E>;

#[derive(Debug, Clone)]
pub struct CratesError(String);

impl CratesError {
    pub(super) fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl fmt::Display for CratesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<reqwest::Error> for CratesError {
    fn from(value: reqwest::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<serde_json::Error> for CratesError {
    fn from(value: serde_json::Error) -> Self {
        Self(value.to_string())
    }
}

use std::num::{ParseFloatError, ParseIntError};

use thiserror::Error;

pub type ParserResult<T, E = ParserError> = Result<T, E>;

#[derive(Default, Debug, Error, Clone, PartialEq)]
pub enum ParserError {
    #[error("incomplete string")]
    IncompleteString,
    #[error("failed to parse integer - {0}")]
    ParseInteger(ParseIntError),
    #[error("failed to parse float - {0}")]
    ParseFloat(ParseFloatError),
    #[error("external error - {0}")]
    ExternalError(String),
    #[error("unknown error")]
    #[default]
    UnknownError,
}

impl ParserError {
    pub(crate) fn external(reason: impl Into<String>) -> Self {
        Self::ExternalError(reason.into())
    }
}

impl From<ParseIntError> for ParserError {
    fn from(e: ParseIntError) -> Self {
        ParserError::ParseInteger(e)
    }
}

impl From<ParseFloatError> for ParserError {
    fn from(e: ParseFloatError) -> Self {
        ParserError::ParseFloat(e)
    }
}

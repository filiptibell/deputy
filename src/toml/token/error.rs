use std::num::{ParseFloatError, ParseIntError};

use thiserror::Error;

pub type TokenizerResult<T, E = TokenizerError> = Result<T, E>;

#[derive(Default, Debug, Error, Clone, PartialEq)]
pub enum TokenizerError {
    #[error("incomplete string")]
    IncompleteString,
    #[error("failed to parse integer - {0}")]
    ParseInteger(#[from] ParseIntError),
    #[error("failed to parse float - {0}")]
    ParseFloat(#[from] ParseFloatError),
    #[error("unknown error")]
    #[default]
    UnknownError,
}

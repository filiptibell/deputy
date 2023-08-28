use thiserror::Error;

pub type ParserResult<T, E = ParserError> = Result<T, E>;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("failed to parse toml - {0}")]
    ParsingFailed(String),
    #[error("external error - {0}")]
    External(String),
}

impl ParserError {
    pub(crate) fn parsing_failed(reason: impl Into<String>) -> Self {
        Self::ParsingFailed(reason.into())
    }

    pub(crate) fn external(reason: impl Into<String>) -> Self {
        Self::External(reason.into())
    }
}

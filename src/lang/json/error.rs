use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsonError {
    #[error("{0}")]
    ParsingError(String),
}

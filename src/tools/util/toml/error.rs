use taplo::dom::Error as ParsingError;
use taplo::parser::Error as SyntaxError;

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum TomlError {
    #[error("{0}")]
    SyntaxError(#[from] SyntaxError),
    #[error("{0}")]
    ParsingError(#[from] ParsingError),
}

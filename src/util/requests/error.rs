use std::{fmt, string::FromUtf8Error};

use thiserror::Error;

pub type RequestResult<T, E = RequestError> = Result<T, E>;

#[derive(Debug, Clone, Error)]
pub struct ResponseError {
    pub(super) status: u16,
    pub(super) bytes: Vec<u8>,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = String::from_utf8(self.bytes.to_vec()) {
            write!(f, "{} - {s}", self.status)
        } else {
            write!(f, "{} - N/A", self.status)
        }
    }
}

#[derive(Debug, Default, Clone, Error)]
pub enum RequestError {
    #[error("utf8 error - {0}")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("request failed - {0}")]
    Response(#[from] ResponseError),
    #[error("failed to parse url - {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("client error - {0}")]
    Client(String), // Request builder error, before sending
    #[error("json error - {0}")]
    Json(String),
    #[error("error - {0}")]
    Custom(String),
    #[error("unknown error")]
    #[default]
    Unknown,
}

impl RequestError {
    pub fn is_not_found_error(&self) -> bool {
        if let RequestError::Response(e) = self {
            e.status == 404
        } else {
            false
        }
    }

    pub fn is_rate_limit_error(&self) -> bool {
        if let RequestError::Response(e) = self {
            if e.status == 429 {
                true
            } else {
                let message = String::from_utf8_lossy(&e.bytes).to_ascii_lowercase();
                message.contains("rate limit exceeded")
                    || message.contains("higher rate limit")
                    || message.contains("#rate-limiting")
            }
        } else {
            false
        }
    }
}

impl From<ureq::Error> for RequestError {
    fn from(value: ureq::Error) -> Self {
        Self::Client(value.to_string())
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}

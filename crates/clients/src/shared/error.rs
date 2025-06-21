use std::{fmt, string::FromUtf8Error};

use reqwest::StatusCode;
use thiserror::Error;

pub type RequestResult<T, E = RequestError> = Result<T, E>;

#[derive(Clone, Error)]
pub struct ResponseError {
    pub(super) status: StatusCode,
    pub(super) bytes: Vec<u8>,
}

impl ResponseError {
    pub fn from_status_and_string(status: StatusCode, string: impl AsRef<str>) -> Self {
        Self {
            status,
            bytes: string.as_ref().as_bytes().to_vec(),
        }
    }
}

impl fmt::Debug for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = String::from_utf8(self.bytes.clone()) {
            f.debug_struct("ResponseError")
                .field("status", &self.status)
                .field("bytes", &s)
                .finish()
        } else {
            f.debug_struct("ResponseError")
                .field("status", &self.status)
                .field("bytes", &"Vec<u8>")
                .finish()
        }
    }
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = String::from_utf8(self.bytes.clone()) {
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
    Client(String),
    #[error("json error - {0}")]
    Json(String),
    #[error("unknown error")]
    #[default]
    Unknown,
}

impl RequestError {
    pub fn is_not_found_error(&self) -> bool {
        if let RequestError::Response(e) = self {
            e.status == StatusCode::NOT_FOUND
        } else {
            false
        }
    }

    pub fn is_rate_limit_error(&self) -> bool {
        if let RequestError::Response(e) = self {
            if e.status == StatusCode::TOO_MANY_REQUESTS {
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

impl From<reqwest::Error> for RequestError {
    fn from(value: reqwest::Error) -> Self {
        Self::Client(value.to_string())
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}

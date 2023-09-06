use std::{fmt, string::FromUtf8Error};

use bytes::Bytes;
use reqwest::StatusCode;
use thiserror::Error;

pub type RequestResult<T, E = RequestError> = Result<T, E>;

#[derive(Debug, Clone, Error)]
pub struct ResponseError {
    status: StatusCode,
    bytes: Bytes,
}

impl From<(StatusCode, Bytes)> for ResponseError {
    fn from((status, bytes): (StatusCode, Bytes)) -> Self {
        Self { status, bytes }
    }
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} - {}",
            self.status.as_u16(),
            self.status.canonical_reason().unwrap(),
            String::from_utf8_lossy(&self.bytes)
        )
    }
}

#[derive(Debug, Default, Clone, Error)]
pub enum RequestError {
    #[error("utf8 error - {0}")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("request failed - {0}")]
    Response(#[from] ResponseError),
    #[error("client error - {0}")]
    Client(String), // Reqwest error before sending
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

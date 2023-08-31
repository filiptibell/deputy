use std::fmt;

pub type GithubResult<T, E = GithubError> = Result<T, E>;

#[derive(Debug, Clone)]
pub struct GithubError(String);

impl fmt::Display for GithubError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<octocrab::Error> for GithubError {
    fn from(value: octocrab::Error) -> Self {
        Self(value.to_string())
    }
}

fn is_not_found_error(error: impl AsRef<str>) -> bool {
    let message = error.as_ref().to_ascii_lowercase();
    message.contains("404") || message.contains("not found")
}

fn is_rate_limit_error(error: impl AsRef<str>) -> bool {
    let message = error.as_ref().to_ascii_lowercase();
    message.contains("rate limit exceeded")
        || message.contains("higher rate limit")
        || message.contains("#rate-limiting")
}

pub trait GithubErrorExt {
    fn is_not_found_error(&self) -> bool;
    fn is_rate_limit_error(&self) -> bool;
}

impl GithubErrorExt for GithubError {
    fn is_not_found_error(&self) -> bool {
        is_not_found_error(&self.0)
    }
    fn is_rate_limit_error(&self) -> bool {
        is_rate_limit_error(&self.0)
    }
}

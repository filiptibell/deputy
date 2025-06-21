use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use tracing::error;

use crate::shared::{Request, RequestResult};

mod cache;
use cache::GithubCache;

use self::consts::{GITHUB_API_VERSION_NAME, GITHUB_API_VERSION_VALUE};

mod consts;
mod requests;

pub mod models;

#[derive(Debug, Clone)]
pub struct GithubClient {
    auth_token: Arc<Mutex<Option<String>>>,
    cache: GithubCache,
    rate_limited: Arc<AtomicBool>,
}

impl GithubClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            auth_token: Arc::new(Mutex::new(None)),
            cache: GithubCache::new(),
            rate_limited: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn request_get(&self, url: impl Into<String>) -> RequestResult<Vec<u8>> {
        let auth_token = self.auth_token.lock().unwrap().clone();

        Request::get(url)
            .with_header("Content-Type", consts::GITHUB_API_CONTENT_TYPE)
            .with_header(GITHUB_API_VERSION_NAME, GITHUB_API_VERSION_VALUE)
            .with_header_opt("Authorization", auth_token)
            .send()
            .await
    }

    fn emit_result<T>(&self, result: &RequestResult<T>) {
        if let Err(e) = &result {
            if e.is_rate_limit_error() {
                self.rate_limited.store(true, Ordering::SeqCst);
            } else {
                error!("GitHub error: {e}");
            }
        }
    }

    #[must_use]
    pub fn is_rate_limited(&self) -> bool {
        self.rate_limited.load(Ordering::SeqCst)
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn set_auth_token(&self, token: impl AsRef<str>) {
        let mut auth_token = self
            .auth_token
            .try_lock()
            .expect("Failed to lock GitHub client");
        *auth_token = Some(format!("Bearer {}", token.as_ref()));

        self.cache.invalidate();

        if self.is_rate_limited() {
            self.rate_limited.store(false, Ordering::SeqCst);
        }
    }
}

impl Default for GithubClient {
    fn default() -> Self {
        Self::new()
    }
}

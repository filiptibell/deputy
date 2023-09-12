use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use surf::Client;
use tracing::error;

use crate::util::*;

mod cache;
use cache::*;

mod consts;
mod requests;

pub mod models;

#[derive(Debug, Clone)]
pub struct GithubClient {
    surf: Arc<Mutex<Client>>,
    surf_auth: Arc<Mutex<Option<String>>>,
    cache: GithubCache,
    rate_limited: Arc<AtomicBool>,
}

impl GithubClient {
    pub fn new(surf: Client) -> Self {
        Self {
            surf: Arc::new(Mutex::new(surf)),
            surf_auth: Arc::new(Mutex::new(None)),
            cache: GithubCache::new(),
            rate_limited: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn request_get(&self, url: impl Into<String>) -> RequestResult<Vec<u8>> {
        let agent = self.surf.lock().unwrap().clone();
        let agent_auth = self.surf_auth.lock().unwrap().clone();

        Request::get(url)
            .with_header("Content-Type", consts::GITHUB_API_CONTENT_TYPE)
            .with_header_opt("Authorization", agent_auth)
            .send(&agent)
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

    pub fn is_rate_limited(&self) -> bool {
        self.rate_limited.load(Ordering::SeqCst)
    }

    pub fn set_auth_token(&self, token: impl AsRef<str>) {
        let mut client_auth = self
            .surf_auth
            .try_lock()
            .expect("Failed to lock GitHub client");
        *client_auth = Some(format!("Bearer {}", token.as_ref()));

        self.cache.invalidate();

        if self.is_rate_limited() {
            self.rate_limited.store(false, Ordering::SeqCst);
        }
    }
}

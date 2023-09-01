use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use octocrab::Octocrab;
use tokio::sync::broadcast;
use tracing::error;

mod cache;
use cache::*;

mod errors;
mod repository;

pub use errors::*;

#[derive(Debug, Clone)]
pub struct GithubWrapper {
    client: Arc<Mutex<octocrab::Octocrab>>,
    cache: GithubCache,
    rate_limit_tx: broadcast::Sender<()>,
    rate_limited: Arc<AtomicBool>,
}

impl GithubWrapper {
    pub fn new() -> Self {
        Self::default()
    }

    fn client(&self) -> Octocrab {
        self.client.lock().unwrap().clone()
    }

    fn emit_result<T>(&self, result: &GithubResult<T>) {
        if let Err(e) = &result {
            if e.is_rate_limit_error() {
                if !self.is_rate_limited() {
                    self.rate_limited.store(true, Ordering::SeqCst);
                    self.rate_limit_tx.send(()).ok();
                }
            } else {
                error!("GitHub error: {e}");
            }
        }
    }

    fn is_rate_limited(&self) -> bool {
        self.rate_limited.load(Ordering::SeqCst)
    }

    pub async fn wait_until_rate_limited_changes(&self) -> bool {
        let mut rate_limit_rx = self.rate_limit_tx.subscribe();
        rate_limit_rx.recv().await.ok();
        self.is_rate_limited()
    }

    pub fn set_auth_token(&self, token: impl Into<String>) {
        let client = octocrab::Octocrab::builder()
            .personal_token(token.into())
            .build()
            .expect("Failed to create GitHub client");

        let mut current_client = self
            .client
            .try_lock()
            .expect("Failed to lock GitHub client");
        *current_client = client;

        self.cache.invalidate();

        if self.is_rate_limited() {
            self.rate_limited.store(false, Ordering::SeqCst);
            self.rate_limit_tx.send(()).ok();
        }
    }
}

impl Default for GithubWrapper {
    fn default() -> Self {
        let client = octocrab::Octocrab::builder()
            .build()
            .expect("Failed to create GitHub client");
        let (rate_limit_tx, _) = broadcast::channel(32);
        Self {
            client: Arc::new(Mutex::new(client)),
            cache: GithubCache::new(),
            rate_limit_tx,
            rate_limited: Arc::new(AtomicBool::new(false)),
        }
    }
}

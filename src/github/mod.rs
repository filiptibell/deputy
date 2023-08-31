use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

mod cache;
use cache::*;
use tokio::sync::{broadcast, Mutex as AsyncMutex};

mod repository;

pub type GithubResult<T, E = GithubError> = Result<T, E>;

#[derive(Debug, Clone)]
pub struct GithubError(String);

impl From<octocrab::Error> for GithubError {
    fn from(value: octocrab::Error) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct GithubWrapper {
    client: Arc<AsyncMutex<octocrab::Octocrab>>,
    cache: GithubCache,
    rate_limit_tx: broadcast::Sender<()>,
    rate_limited: Arc<AtomicBool>,
}

impl GithubWrapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bust_cache(&self) {
        self.cache.invalidate()
    }

    pub fn is_rate_limited(&self) -> bool {
        self.rate_limited.load(Ordering::SeqCst)
    }

    pub fn notify_rate_limited(&self) -> bool {
        if !self.is_rate_limited() {
            self.rate_limited.store(true, Ordering::SeqCst);
            self.rate_limit_tx.send(()).ok();
            true
        } else {
            false
        }
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

        self.bust_cache();
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
            client: Arc::new(AsyncMutex::new(client)),
            cache: GithubCache::new(),
            rate_limit_tx,
            rate_limited: Arc::new(AtomicBool::new(false)),
        }
    }
}

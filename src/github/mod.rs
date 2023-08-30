use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

mod cache;
use cache::*;
use tokio::sync::broadcast;

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
    client: Arc<octocrab::Octocrab>,
    cache: GithubCache,
    rate_limit_tx: broadcast::Sender<()>,
    rate_limited: Arc<AtomicBool>,
}

impl GithubWrapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bust_cache(&self) {
        self.cache.bust_all()
    }

    pub fn notify_rate_limited(&self) -> bool {
        if !self.rate_limited.load(Ordering::SeqCst) {
            self.rate_limited.store(true, Ordering::SeqCst);
            self.rate_limit_tx.send(()).ok();
            true
        } else {
            false
        }
    }

    pub fn reset_rate_limited(&self) -> bool {
        if self.rate_limited.load(Ordering::SeqCst) {
            self.rate_limited.store(false, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub async fn wait_until_rate_limited(&self) -> bool {
        if self.rate_limited.load(Ordering::SeqCst) {
            return true;
        }
        let rate_limited = Arc::clone(&self.rate_limited);
        let mut rate_limit_rx = self.rate_limit_tx.subscribe();
        rate_limit_rx.recv().await.ok();
        rate_limited.load(Ordering::SeqCst)
    }
}

impl Default for GithubWrapper {
    fn default() -> Self {
        let client = octocrab::Octocrab::builder()
            .build()
            .expect("Failed to create GitHub client");
        let (rate_limit_tx, _) = broadcast::channel(32);
        Self {
            client: Arc::new(client),
            cache: GithubCache::new(),
            rate_limit_tx,
            rate_limited: Arc::new(AtomicBool::new(false)),
        }
    }
}

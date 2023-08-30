use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::Mutex;
use tracing::trace;

use octocrab::models::{repos::Release, RepositoryMetrics};

use super::GithubResult;

/*
    Cache durations for requests:

    * Error, non-important request - 10 minutes
    * Error, important request - 1 minute
    * Success, non-important request - 30 minutes
    * Success, important request - 5 minutes
*/
const ERR_CACHE_DURATION: Duration = Duration::from_secs(60 * 10);
const ERR_CACHE_DURATION_IMPORTANT: Duration = Duration::from_secs(60);
const OK_CACHE_DURATION: Duration = Duration::from_secs(60 * 30);
const OK_CACHE_DURATION_IMPORTANT: Duration = Duration::from_secs(60 * 5);

type CacheMap<T> = Arc<Mutex<HashMap<String, GithubResult<T>>>>;

#[derive(Debug, Default, Clone)]
pub(super) struct GithubCache {
    pub repository_metrics: CacheMap<RepositoryMetrics>,
    pub latest_releases: CacheMap<Release>,
}

impl GithubCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bust_all(&self) {
        self.bust_one(&self.repository_metrics);
        self.bust_one(&self.latest_releases);
    }

    pub fn bust_one<T>(&self, cache: &CacheMap<T>)
    where
        T: Clone + Send + 'static,
    {
        cache
            .try_lock()
            .expect("Failed to lock cache for busting")
            .clear()
    }

    pub async fn get<T>(
        &self,
        cache: &CacheMap<T>,
        cache_key: impl AsRef<str>,
    ) -> Option<GithubResult<T>>
    where
        T: Clone + Send + 'static,
    {
        let cache_key = cache_key.as_ref();
        let cache_guard = cache.lock().await;
        if cache_guard.contains_key(cache_key) {
            trace!("Cache hit on key '{cache_key}'");
        }
        cache_guard.get(cache_key).cloned()
    }

    pub async fn add<T>(
        &self,
        cache: &CacheMap<T>,
        cache_key: impl Into<String>,
        value: GithubResult<T>,
        important: bool,
    ) -> GithubResult<T>
    where
        T: Clone + Send + 'static,
    {
        let cache = cache.clone();
        let cache_key = cache_key.into();

        cache.lock().await.insert(cache_key.clone(), value.clone());

        let duration = match (value.is_ok(), important) {
            (false, false) => ERR_CACHE_DURATION,
            (false, true) => ERR_CACHE_DURATION_IMPORTANT,
            (true, false) => OK_CACHE_DURATION,
            (true, true) => OK_CACHE_DURATION_IMPORTANT,
        };

        tokio::task::spawn(async move {
            tokio::time::sleep(duration).await;
            cache.lock().await.remove(&cache_key);
        });

        value
    }
}

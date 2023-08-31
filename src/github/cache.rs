use std::{hash::Hash, time::Duration};

use moka::future::Cache;

use octocrab::models::{repos::Release, RepositoryMetrics};

use super::GithubResult;

type CacheMap<T> = Cache<String, GithubResult<T>>;

#[derive(Debug, Clone)]
pub(super) struct GithubCache {
    pub repository_metrics: CacheMap<RepositoryMetrics>,
    pub repository_releases: CacheMap<Vec<Release>>,
}

impl GithubCache {
    pub fn new() -> Self {
        Self {
            repository_metrics: new_cache(60, 15),
            repository_releases: new_cache(30, 5),
        }
    }

    pub fn invalidate(&self) {
        self.repository_metrics.invalidate_all();
        self.repository_releases.invalidate_all();
    }
}

fn new_cache<K, V>(minutes_to_live: u64, minutes_to_idle: u64) -> Cache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    Cache::builder()
        .max_capacity(64)
        .time_to_live(Duration::from_secs(60 * minutes_to_live))
        .time_to_idle(Duration::from_secs(60 * minutes_to_idle))
        .build()
}

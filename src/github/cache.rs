use crate::util::*;

use super::models::*;
use super::*;

#[derive(Debug, Clone)]
pub(super) struct GithubCache {
    pub repository_metrics: RequestCacheMap<GithubResult<RepositoryMetrics>>,
    pub repository_releases: RequestCacheMap<GithubResult<Vec<RepositoryRelease>>>,
}

impl GithubCache {
    pub fn new() -> Self {
        Self {
            repository_metrics: RequestCacheMap::new(60, 15),
            repository_releases: RequestCacheMap::new(30, 5),
        }
    }

    pub fn invalidate(&self) {
        self.repository_metrics.invalidate();
        self.repository_releases.invalidate();
    }
}

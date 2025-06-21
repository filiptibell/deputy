use crate::util::{RequestCacheMap, RequestResult};

use super::models::{GitTreeRoot, RepositoryMetrics, RepositoryRelease};

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone)]
pub(super) struct GithubCache {
    pub repository_metrics: RequestCacheMap<RequestResult<RepositoryMetrics>>,
    pub repository_releases: RequestCacheMap<RequestResult<Vec<RepositoryRelease>>>,
    pub repository_trees: RequestCacheMap<RequestResult<GitTreeRoot>>,
    pub repository_files: RequestCacheMap<RequestResult<Vec<u8>>>,
}

impl GithubCache {
    pub fn new() -> Self {
        Self {
            repository_metrics: RequestCacheMap::new(60, 15),
            repository_releases: RequestCacheMap::new(30, 5),
            repository_trees: RequestCacheMap::new(45, 10),
            repository_files: RequestCacheMap::new(10, 5),
        }
    }

    pub fn invalidate(&self) {
        self.repository_metrics.invalidate();
        self.repository_releases.invalidate();
        self.repository_trees.invalidate();
        self.repository_files.invalidate();
    }
}

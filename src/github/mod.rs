use std::sync::Arc;

mod cache;
use cache::*;

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
}

impl GithubWrapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bust_cache(&self) {
        self.cache.bust_all()
    }
}

impl Default for GithubWrapper {
    fn default() -> Self {
        let client = octocrab::Octocrab::builder()
            .build()
            .expect("Failed to create GitHub client");
        Self {
            client: Arc::new(client),
            cache: GithubCache::new(),
        }
    }
}

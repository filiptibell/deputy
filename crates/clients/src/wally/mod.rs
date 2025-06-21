use super::github::GithubClient;
use crate::shared::{RequestError, RequestResult, ResponseError};

mod cache;
use cache::WallyCache;

mod requests;

pub mod models;

#[derive(Debug, Clone)]
pub struct WallyClient {
    cache: WallyCache,
    github: GithubClient,
}

impl WallyClient {
    #[must_use]
    pub fn new(github: GithubClient) -> Self {
        Self {
            cache: WallyCache::new(),
            github,
        }
    }
}

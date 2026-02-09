use crate::shared::{RequestError, RequestResult, ResponseError};

use super::github::GithubClient;

mod cache;
mod requests;

pub mod models;

use self::cache::WallyCache;

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

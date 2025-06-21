use super::github::GithubClient;
use crate::util::{RequestError, RequestResult, ResponseError};

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
    pub fn new(github: GithubClient) -> Self {
        Self {
            cache: WallyCache::new(),
            github,
        }
    }
}

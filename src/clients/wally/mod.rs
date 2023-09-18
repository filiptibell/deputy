use super::github::*;
use crate::util::*;

mod cache;
use cache::*;

mod requests;

pub mod models;

#[derive(Debug, Clone)]
pub struct WallyClient {
    _cache: WallyCache,
    github: GithubClient,
}

impl WallyClient {
    pub fn new(github: GithubClient) -> Self {
        Self {
            _cache: WallyCache::new(),
            github,
        }
    }
}

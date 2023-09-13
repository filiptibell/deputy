use surf::Client;

use super::github::*;
use crate::util::*;

mod cache;
use cache::*;

mod requests;

pub mod models;

#[derive(Debug, Clone)]
pub struct WallyClient {
    _surf: Client,
    _cache: WallyCache,
    github: GithubClient,
}

impl WallyClient {
    pub fn new(surf: Client, github: GithubClient) -> Self {
        Self {
            _surf: surf,
            _cache: WallyCache::new(),
            github,
        }
    }
}

use surf::Client;
use tracing::error;

use super::github::*;
use crate::util::*;

mod cache;
use cache::*;

mod requests;

pub mod models;

#[derive(Debug, Clone)]
pub struct WallyClient {
    surf: Client,
    cache: WallyCache,
    github: GithubClient,
}

impl WallyClient {
    pub fn new(surf: Client, github: GithubClient) -> Self {
        Self {
            surf,
            cache: WallyCache::new(),
            github,
        }
    }

    async fn request_get(&self, url: impl Into<String>) -> RequestResult<Vec<u8>> {
        Request::get(url).send(&self.surf).await
    }

    fn emit_result<T>(&self, result: &RequestResult<T>) {
        if let Err(e) = &result {
            error!("Wally error: {e}");
        }
    }
}

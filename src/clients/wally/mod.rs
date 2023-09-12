use tracing::error;
use ureq::Agent;

use super::github::*;
use crate::util::*;

mod cache;
use cache::*;

mod requests;

pub mod models;

#[derive(Debug, Clone)]
pub struct WallyClient {
    agent: Agent,
    cache: WallyCache,
    github: GithubClient,
}

impl WallyClient {
    pub fn new(agent: Agent, github: GithubClient) -> Self {
        Self {
            agent,
            cache: WallyCache::new(),
            github,
        }
    }

    async fn request_get(&self, url: impl Into<String>) -> RequestResult<Vec<u8>> {
        Request::get(url).send(&self.agent).await
    }

    fn emit_result<T>(&self, result: &RequestResult<T>) {
        if let Err(e) = &result {
            error!("Wally error: {e}");
        }
    }
}

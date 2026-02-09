use tracing::error;

use crate::shared::{Request, RequestResult};

use super::github::GithubClient;

mod cache;
mod consts;
mod requests;
mod util;

pub mod models;

use self::cache::GolangCache;

#[derive(Debug, Clone)]
pub struct GolangClient {
    cache: GolangCache,
    github: GithubClient,
}

impl GolangClient {
    #[must_use]
    pub fn new(github: GithubClient) -> Self {
        Self {
            cache: GolangCache::new(),
            github,
        }
    }

    async fn request_get(&self, url: impl Into<String>) -> RequestResult<Vec<u8>> {
        Request::get(url).send().await
    }

    fn emit_result<T>(result: &RequestResult<T>) {
        if let Err(e) = &result {
            error!("Golang error: {e}");
        }
    }
}

impl Default for GolangClient {
    fn default() -> Self {
        Self::new(GithubClient::new())
    }
}

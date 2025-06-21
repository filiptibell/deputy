use tracing::error;

use crate::shared::{Request, RequestResult};

mod cache;
use cache::NpmCache;

mod consts;
mod requests;

pub mod models;

#[derive(Debug, Clone)]
pub struct NpmClient {
    cache: NpmCache,
}

impl NpmClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: NpmCache::new(),
        }
    }

    async fn request_get(&self, url: impl Into<String>) -> RequestResult<Vec<u8>> {
        Request::get(url).send().await
    }

    fn emit_result<T>(result: &RequestResult<T>) {
        if let Err(e) = &result {
            error!("NPM error: {e}");
        }
    }
}

impl Default for NpmClient {
    fn default() -> Self {
        Self::new()
    }
}

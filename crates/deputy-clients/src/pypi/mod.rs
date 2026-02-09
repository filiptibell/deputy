use tracing::error;

use crate::shared::{Request, RequestResult};

mod cache;
mod consts;
mod requests;
mod util;

pub mod models;

use self::cache::PyPiCache;

#[derive(Debug, Clone)]
pub struct PyPiClient {
    cache: PyPiCache,
}

impl PyPiClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: PyPiCache::new(),
        }
    }

    async fn request_get(&self, url: impl Into<String>) -> RequestResult<Vec<u8>> {
        Request::get(url).send().await
    }

    fn emit_result<T>(result: &RequestResult<T>) {
        if let Err(e) = &result {
            error!("PyPI error: {e}");
        }
    }
}

impl Default for PyPiClient {
    fn default() -> Self {
        Self::new()
    }
}

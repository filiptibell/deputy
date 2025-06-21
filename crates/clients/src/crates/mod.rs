use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use tokio::{
    sync::broadcast::{Sender, channel},
    time::sleep,
};
use tracing::error;

use crate::shared::{Request, RequestError, RequestResult};

mod cache;
use cache::CratesCache;

mod consts;
mod requests;

pub mod models;

#[derive(Debug, Clone)]
pub struct CratesClient {
    cache: CratesCache,
    crawl_channel: Sender<()>,
    crawl_limited: Arc<AtomicBool>,
}

impl CratesClient {
    #[must_use]
    pub fn new() -> Self {
        let crawl_channel = channel(1).0;
        Self {
            cache: CratesCache::new(),
            crawl_channel,
            crawl_limited: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn request_get(&self, url: impl Into<String>) -> RequestResult<Vec<u8>> {
        Request::get(url).send().await
    }

    fn emit_result<T>(result: &RequestResult<T>) {
        if let Err(e) = &result {
            error!("Crates error: {e}");
        }
    }

    fn is_crawl_limited(&self) -> bool {
        self.crawl_limited.load(Ordering::SeqCst)
    }

    fn set_crawl_limited(&self) {
        if !self.is_crawl_limited() {
            let lim = self.crawl_limited.clone();
            let tx = self.crawl_channel.clone();
            lim.store(true, Ordering::SeqCst);
            tokio::spawn(async move {
                sleep(Duration::from_secs_f32(consts::CRAWL_MAX_INTERVAL_SECONDS)).await;
                lim.store(false, Ordering::SeqCst);
                tx.send(()).ok();
            });
        }
    }

    async fn wait_for_crawl_limit(&self) {
        if self.is_crawl_limited() {
            let mut rx = self.crawl_channel.subscribe();
            rx.recv().await.ok();
        }
    }
}

impl Default for CratesClient {
    fn default() -> Self {
        Self::new()
    }
}

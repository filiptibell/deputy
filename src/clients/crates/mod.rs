use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use async_broadcast::{broadcast, Sender};
use smol::Timer;
use tracing::error;

use ureq::Agent;

use crate::util::*;

mod cache;
use cache::*;

mod consts;
mod requests;

pub mod models;

#[derive(Debug, Clone)]
pub struct CratesClient {
    agent: Agent,
    cache: CratesCache,
    crawl_limit_tx: Sender<()>,
    crawl_limited: Arc<AtomicBool>,
}

impl CratesClient {
    pub fn new(agent: Agent) -> Self {
        let (crawl_limit_tx, _) = broadcast(32);
        Self {
            agent,
            cache: CratesCache::new(),
            crawl_limit_tx,
            crawl_limited: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn request_get(&self, url: impl Into<String>) -> RequestResult<Vec<u8>> {
        Request::get(url).send(&self.agent).await
    }

    fn emit_result<T>(&self, result: &RequestResult<T>) {
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
            let tx = self.crawl_limit_tx.clone();
            lim.store(true, Ordering::SeqCst);
            smol::spawn(async move {
                Timer::after(Duration::from_secs(consts::CRAWL_MAX_INTERVAL_SECONDS)).await;
                lim.store(false, Ordering::SeqCst);
                tx.try_broadcast(()).ok();
            })
            .detach();
        }
    }

    async fn wait_for_crawl_limit(&self) {
        if self.is_crawl_limited() {
            let mut crawl_limit_rx = self.crawl_limit_tx.new_receiver();
            crawl_limit_rx.recv().await.ok();
        }
    }
}

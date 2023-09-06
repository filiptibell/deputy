use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use bytes::Bytes;
use tokio::{sync::broadcast, time::sleep};
use tracing::error;

use reqwest::{Client, Method, StatusCode};

use crate::util::*;

mod cache;
use cache::*;

mod consts;
mod models;
mod requests;

#[derive(Debug, Clone)]
pub struct CratesWrapper {
    client: Arc<Mutex<Client>>,
    cache: CratesCache,
    crawl_limit_tx: broadcast::Sender<()>,
    crawl_limited: Arc<AtomicBool>,
}

impl CratesWrapper {
    pub fn new(client: Client) -> Self {
        let (crawl_limit_tx, _) = broadcast::channel(32);
        Self {
            client: Arc::new(Mutex::new(client)),
            cache: CratesCache::new(),
            crawl_limit_tx,
            crawl_limited: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn request(
        &self,
        method: Method,
        url: impl Into<String>,
    ) -> Result<(StatusCode, Bytes), reqwest::Error> {
        let client = self.client.lock().unwrap().clone();

        let response = client.request(method, url.into()).send().await?;
        let status = response.status();
        let bytes = response.bytes().await?;

        Ok((status, bytes))
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
            tokio::spawn(async move {
                sleep(Duration::from_secs(consts::CRAWL_MAX_INTERVAL_SECONDS)).await;
                lim.store(false, Ordering::SeqCst);
                tx.send(()).ok();
            });
        }
    }

    async fn wait_for_crawl_limit(&self) {
        if self.is_crawl_limited() {
            let mut crawl_limit_rx = self.crawl_limit_tx.subscribe();
            crawl_limit_rx.recv().await.ok();
        }
    }
}

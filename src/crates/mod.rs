use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use reqwest::{
    header::{HeaderMap, HeaderValue, USER_AGENT},
    Client,
};
use tokio::{sync::broadcast, time::sleep};
use tracing::error;

mod cache;
use cache::*;

mod crates_io;
mod errors;
mod index;
mod requests;

pub use errors::*;

use self::requests::{CRAWL_MAX_INTERVAL_SECONDS, CRAWL_USER_AGENT_VALUE};

#[derive(Debug, Clone)]
pub struct CratesWrapper {
    client: Arc<Mutex<Client>>,
    cache: CratesCache,
    crawl_limit_tx: broadcast::Sender<()>,
    crawl_limited: Arc<AtomicBool>,
}

impl CratesWrapper {
    pub fn new() -> Self {
        Self::default()
    }

    fn client(&self) -> Client {
        self.client.lock().unwrap().clone()
    }

    fn emit_result<T>(&self, result: &CratesResult<T>) {
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
                sleep(Duration::from_secs(CRAWL_MAX_INTERVAL_SECONDS)).await;
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

impl Default for CratesWrapper {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(CRAWL_USER_AGENT_VALUE));
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create crates client");
        let (crawl_limit_tx, _) = broadcast::channel(32);
        Self {
            client: Arc::new(Mutex::new(client)),
            cache: CratesCache::new(),
            crawl_limit_tx,
            crawl_limited: Arc::new(AtomicBool::new(false)),
        }
    }
}

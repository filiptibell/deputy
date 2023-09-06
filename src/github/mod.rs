use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use bytes::Bytes;
use tokio::sync::broadcast;
use tracing::error;

use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
    Client, Method, StatusCode,
};

use crate::util::*;

mod cache;
use cache::*;

mod models;
mod requests;

use self::requests::{GITHUB_API_CONTENT_TYPE, GITHUB_API_USER_AGENT};

#[derive(Debug, Clone)]
pub struct GithubWrapper {
    client: Arc<Mutex<Client>>,
    client_auth: Arc<Mutex<Option<String>>>,
    cache: GithubCache,
    rate_limit_tx: broadcast::Sender<()>,
    rate_limited: Arc<AtomicBool>,
}

impl GithubWrapper {
    pub fn new() -> Self {
        Self::default()
    }

    async fn request(
        &self,
        method: Method,
        url: impl Into<String>,
    ) -> Result<(StatusCode, Bytes), reqwest::Error> {
        let client = self.client.lock().unwrap().clone();
        let client_auth = self.client_auth.lock().unwrap().clone();

        let mut request = client
            .request(method, url.into())
            .header(USER_AGENT, HeaderValue::from_static(GITHUB_API_USER_AGENT))
            .header(
                CONTENT_TYPE,
                HeaderValue::from_static(GITHUB_API_CONTENT_TYPE),
            );
        if let Some(auth) = client_auth {
            request = request.header(AUTHORIZATION, &auth);
        }

        let response = request.send().await?;
        let status = response.status();
        let bytes = response.bytes().await?;

        Ok((status, bytes))
    }

    fn emit_result<T>(&self, result: &RequestResult<T>) {
        if let Err(e) = &result {
            if e.is_rate_limit_error() {
                if !self.is_rate_limited() {
                    self.rate_limited.store(true, Ordering::SeqCst);
                    self.rate_limit_tx.send(()).ok();
                }
            } else {
                error!("GitHub error: {e}");
            }
        }
    }

    fn is_rate_limited(&self) -> bool {
        self.rate_limited.load(Ordering::SeqCst)
    }

    pub async fn wait_until_rate_limited_changes(&self) -> bool {
        let mut rate_limit_rx = self.rate_limit_tx.subscribe();
        rate_limit_rx.recv().await.ok();
        self.is_rate_limited()
    }

    pub fn set_auth_token(&self, token: impl AsRef<str>) {
        let mut client_auth = self
            .client_auth
            .try_lock()
            .expect("Failed to lock GitHub client");
        *client_auth = Some(format!("Bearer {}", token.as_ref()));

        self.cache.invalidate();

        if self.is_rate_limited() {
            self.rate_limited.store(false, Ordering::SeqCst);
            self.rate_limit_tx.send(()).ok();
        }
    }
}

impl Default for GithubWrapper {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(GITHUB_API_USER_AGENT));
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static(GITHUB_API_CONTENT_TYPE),
        );
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create GitHub client");
        let (rate_limit_tx, _) = broadcast::channel(32);
        Self {
            client: Arc::new(Mutex::new(client)),
            client_auth: Arc::new(Mutex::new(None)),
            cache: GithubCache::new(),
            rate_limit_tx,
            rate_limited: Arc::new(AtomicBool::new(false)),
        }
    }
}

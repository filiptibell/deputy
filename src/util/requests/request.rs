#![allow(dead_code)]

use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use http_types::{
    headers::{HeaderName, HeaderValue, USER_AGENT},
    Method,
};

use futures_lite::future::race;
use smol::Timer;
use tracing::{trace, warn};
use url::Url;

use super::{client::fetch, RequestError, RequestResult, ResponseError};

const USER_AGENT_VALUE: &str = concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"));
static REQUEST_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug)]
pub struct Request {
    method: Method,
    url: String,
    body: Vec<u8>,
    headers: HashMap<String, String>,
}

impl Request {
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            body: Vec::new(),
            headers: HashMap::new(),
        }
    }

    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::Get, url)
    }

    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let mut name: String = name.into();
        name.make_ascii_lowercase();
        self.headers.insert(name, value.into());
        self
    }

    pub fn with_header_opt(
        self,
        name: impl Into<String>,
        value: Option<impl Into<String>>,
    ) -> Self {
        if let Some(value) = value {
            self.with_header(name, value)
        } else {
            self
        }
    }

    pub fn with_headers(mut self, pairs: &[(&str, &str)]) -> Self {
        for (name, value) in pairs {
            self.headers.insert(name.to_string(), value.to_string());
        }
        self
    }

    pub async fn send(self) -> RequestResult<Vec<u8>> {
        let id = REQUEST_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        trace!("Sending request #{id}: {self:#?}");

        let url = Url::parse(&self.url)?;
        let mut request = http_types::Request::new(self.method, url);

        for (key, value) in self.headers {
            request.append_header(
                HeaderName::from_string(key).expect("Passed invalid header name"),
                &value,
            )
        }

        // Force user agent
        request.append_header(
            USER_AGENT,
            HeaderValue::from_bytes(USER_AGENT_VALUE.as_bytes().to_vec())
                .expect("User agent header is invalid"),
        );

        // Send request with manual timeout mechanism
        let mut response = race(async { Ok(fetch(request).await?) }, async {
            Timer::after(Duration::from_secs(10)).await;
            warn!("Request timed out");
            Err(RequestError::TimedOut)
        })
        .await?;

        trace!("Got response #{id}: {response:#?}");

        let status = response.status();
        let body = response
            .body_bytes()
            .await
            .map_err(|e| RequestError::Client(e.to_string()))?;

        if status.is_client_error() || status.is_server_error() {
            let e = ResponseError {
                status,
                bytes: body,
            };
            return Err(e.into());
        }

        Ok(body)
    }
}

#![allow(dead_code)]

use std::{collections::HashMap, time::Duration};

use once_cell::sync::Lazy;
use surf::{
    http::{
        headers::{HeaderName, USER_AGENT},
        Method,
    },
    Client, Config,
};

use super::{RequestResult, ResponseError};

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Config::new()
        .set_max_connections_per_host(8)
        .set_timeout(Some(Duration::from_secs(15)))
        .add_header(
            USER_AGENT,
            concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION")),
        )
        .expect("Failed to add user agent header")
        .try_into()
        .expect("Failed to create surf client")
});

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
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn with_header_opt(
        mut self,
        name: impl Into<String>,
        value: Option<impl Into<String>>,
    ) -> Self {
        if let Some(value) = value {
            self.headers.insert(name.into(), value.into());
        }
        self
    }

    pub fn with_headers(mut self, pairs: &[(&str, &str)]) -> Self {
        for (name, value) in pairs {
            self.headers.insert(name.to_string(), value.to_string());
        }
        self
    }

    pub async fn send(self) -> RequestResult<Vec<u8>> {
        let mut request = match self.method {
            Method::Get => CLIENT.get(&self.url),
            Method::Post => CLIENT.post(&self.url),
            Method::Patch => CLIENT.patch(&self.url),
            Method::Put => CLIENT.put(&self.url),
            Method::Delete => CLIENT.delete(&self.url),
            Method::Head => CLIENT.head(&self.url),
            _ => panic!("Unsupported method"),
        };

        for (key, value) in self.headers {
            request = request.header(
                HeaderName::from_string(key).expect("Passed invalid header name"),
                &value,
            )
        }

        let mut response = request.body_bytes(&self.body).send().await?;
        let status = response.status();
        let body = response.body_bytes().await?;

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

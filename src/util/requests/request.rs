#![allow(dead_code)]

use std::{collections::HashMap, str::FromStr};

use once_cell::sync::Lazy;
use reqwest::{
    header::{HeaderName, HeaderValue, USER_AGENT},
    Client, Method,
};
use tracing::trace;
use url::Url;

use super::{RequestError, RequestResult, ResponseError};

const USER_AGENT_VALUE: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "@",
    env!("CARGO_PKG_VERSION"),
    " ( ",
    env!("CARGO_PKG_REPOSITORY"),
    " )"
);

static CLIENT: Lazy<Client> = Lazy::new(Client::new);

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
        Self::new(Method::GET, url)
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
        let mut request = reqwest::Request::new(self.method, Url::parse(&self.url)?);

        // Set headers
        let headers = request.headers_mut();
        for (key, value) in self.headers {
            headers.insert(
                HeaderName::from_str(&key).expect("Passed invalid header name"),
                HeaderValue::from_str(&value).expect("Passed invalid header value"),
            );
        }

        // Force user agent
        headers.insert(
            USER_AGENT,
            HeaderValue::from_bytes(USER_AGENT_VALUE.as_bytes())
                .expect("User agent header is invalid"),
        );

        // Send request
        trace!("Sending request:\n{request:#?}");
        let response = CLIENT.execute(request).await?;
        trace!("Got response:\n{response:#?}");
        let status = response.status();
        let body = response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| RequestError::Client(e.to_string()))?;

        // Return Err enum if the request has a non-200 status code
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

#![allow(dead_code)]

use std::{collections::HashMap, fmt};

use surf::{http::headers::HeaderName, Client};

use super::{RequestResult, ResponseError};

#[derive(Clone, Copy, Debug, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum Method {
    #[default]
    GET,
    POST,
    PATCH,
    PUT,
    DELETE,
    HEAD,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::PATCH => "PATCH",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
            Self::HEAD => "HEAD",
        };
        s.fmt(f)
    }
}

#[derive(Clone, Debug, Default)]
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
            ..Default::default()
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

    pub async fn send(self, surf: &Client) -> RequestResult<Vec<u8>> {
        let mut request = match self.method {
            Method::GET => surf.get(&self.url),
            Method::POST => surf.post(&self.url),
            Method::PATCH => surf.patch(&self.url),
            Method::PUT => surf.put(&self.url),
            Method::DELETE => surf.delete(&self.url),
            Method::HEAD => surf.head(&self.url),
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

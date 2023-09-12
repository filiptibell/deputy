#![allow(dead_code)]

use std::{collections::HashMap, fmt};

use smol::unblock;
use ureq::Agent;

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

    pub async fn send(self, agent: &Agent) -> RequestResult<Vec<u8>> {
        // TODO: We should probably switch to using something like surf
        // instead of ureq and spawning blocking threads, we can depend
        // on surf with default-features = false + h1-client-rustls
        let agent = agent.clone();

        let mut request = match self.method {
            Method::GET => agent.get(&self.url),
            Method::POST => agent.post(&self.url),
            Method::PATCH => agent.patch(&self.url),
            Method::PUT => agent.put(&self.url),
            Method::DELETE => agent.delete(&self.url),
            Method::HEAD => agent.head(&self.url),
        };

        for (key, value) in self.headers {
            request = request.set(&key, &value)
        }

        let response = unblock(move || request.send_bytes(&self.body)).await?;
        let status = response.status();

        let len: usize = response
            .header("Content-Length")
            .unwrap_or_default()
            .parse()
            .unwrap_or(0);

        let body = unblock(move || {
            let mut bytes = Vec::<u8>::with_capacity(len);
            response
                .into_reader()
                .read_to_end(&mut bytes)
                .map_err(ureq::Error::from)?;
            Ok::<_, ureq::Error>(bytes)
        })
        .await?;

        if status >= 400 {
            let e = ResponseError {
                status,
                bytes: body,
            };
            return Err(e.into());
        }

        Ok(body)
    }
}

use reqwest::{
    header::{HeaderMap, HeaderValue, USER_AGENT},
    Client,
};

pub mod crates;
pub mod github;

use crates::CratesClient;
use github::GithubClient;

#[derive(Debug, Clone)]
pub struct Clients {
    pub crates: CratesClient,
    pub github: GithubClient,
}

impl Clients {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(concat!(
                env!("CARGO_PKG_NAME"),
                "@",
                env!("CARGO_PKG_VERSION")
            )),
        );

        let base = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create reqwest client");

        let github = GithubClient::new(base.clone());
        let crates = CratesClient::new(base.clone());

        Self { crates, github }
    }
}

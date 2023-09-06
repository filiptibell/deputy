use reqwest::{
    header::{HeaderMap, HeaderValue, USER_AGENT},
    Client,
};

pub mod crates;
pub mod github;

pub use crates::CratesWrapper;
pub use github::GithubWrapper;

#[derive(Debug, Clone)]
pub struct Clients {
    pub crates: CratesWrapper,
    pub github: GithubWrapper,
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

        let github = GithubWrapper::new(base.clone());
        let crates = CratesWrapper::new(base.clone());

        Self { crates, github }
    }
}

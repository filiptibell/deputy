use std::time::Duration;

use surf::{http::headers::USER_AGENT, Client, Config};

pub mod crates;
pub mod github;
pub mod wally;

use crates::CratesClient;
use github::GithubClient;
use wally::WallyClient;

#[derive(Debug, Clone)]
pub struct Clients {
    pub crates: CratesClient,
    pub github: GithubClient,
    pub wally: WallyClient,
}

impl Clients {
    pub fn new() -> Self {
        let base: Client = Config::new()
            .set_max_connections_per_host(8)
            .set_timeout(Some(Duration::from_secs(15)))
            .add_header(
                USER_AGENT,
                concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION")),
            )
            .expect("Failed to add user agent header")
            .try_into()
            .expect("Failed to create surf client");

        let crates = CratesClient::new(base.clone());
        let github = GithubClient::new(base.clone());
        let wally = WallyClient::new(base.clone(), github.clone());

        Self {
            crates,
            github,
            wally,
        }
    }
}

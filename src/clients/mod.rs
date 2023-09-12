pub mod crates;
pub mod github;
pub mod wally;

use crates::CratesClient;
use github::GithubClient;
use ureq::AgentBuilder;
use wally::WallyClient;

#[derive(Debug, Clone)]
pub struct Clients {
    pub crates: CratesClient,
    pub github: GithubClient,
    pub wally: WallyClient,
}

impl Clients {
    pub fn new() -> Self {
        let base = AgentBuilder::new()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "@",
                env!("CARGO_PKG_VERSION")
            ))
            .build();

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

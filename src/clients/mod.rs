pub mod crates;
pub mod github;

use crates::CratesClient;
use github::GithubClient;
use ureq::AgentBuilder;

#[derive(Debug, Clone)]
pub struct Clients {
    pub crates: CratesClient,
    pub github: GithubClient,
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

        let github = GithubClient::new(base.clone());
        let crates = CratesClient::new(base.clone());

        Self { crates, github }
    }
}

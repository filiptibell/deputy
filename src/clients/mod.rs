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
        let crates = CratesClient::new();
        let github = GithubClient::new();
        let wally = WallyClient::new(github.clone());

        Self {
            crates,
            github,
            wally,
        }
    }
}

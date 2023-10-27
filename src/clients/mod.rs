pub mod crates;
pub mod github;
pub mod npm;
pub mod wally;

use crates::CratesClient;
use github::GithubClient;
use npm::NpmClient;
use wally::WallyClient;

#[derive(Debug, Clone)]
pub struct Clients {
    pub crates: CratesClient,
    pub github: GithubClient,
    pub npm: NpmClient,
    pub wally: WallyClient,
}

impl Clients {
    pub fn new() -> Self {
        let crates = CratesClient::new();
        let github = GithubClient::new();
        let npm = NpmClient::new();
        let wally = WallyClient::new(github.clone());

        Self {
            crates,
            github,
            npm,
            wally,
        }
    }
}

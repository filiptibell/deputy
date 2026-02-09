#![allow(dead_code)]

mod shared;

pub mod crates;
pub mod github;
pub mod npm;
pub mod pypi;
pub mod wally;

use self::crates::CratesClient;
use self::github::GithubClient;
use self::npm::NpmClient;
use self::pypi::PyPiClient;
use self::wally::WallyClient;

#[derive(Debug, Clone)]
pub struct Clients {
    pub crates: CratesClient,
    pub github: GithubClient,
    pub npm: NpmClient,
    pub pypi: PyPiClient,
    pub wally: WallyClient,
}

impl Clients {
    #[must_use]
    pub fn new() -> Self {
        let crates = CratesClient::new();
        let github = GithubClient::new();
        let npm = NpmClient::new();
        let pypi = PyPiClient::new();
        let wally = WallyClient::new(github.clone());

        Self {
            crates,
            github,
            npm,
            pypi,
            wally,
        }
    }
}

impl Default for Clients {
    fn default() -> Self {
        Self::new()
    }
}

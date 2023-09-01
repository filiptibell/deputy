use tower_lsp::Client;

use crate::github::*;
use crate::server::*;

use super::*;

#[derive(Debug, Clone)]
pub struct Cargo {
    _client: Client,
    _github: GithubWrapper,
    _documents: Documents,
}

impl Cargo {
    pub(super) fn new(client: Client, github: GithubWrapper, documents: Documents) -> Self {
        Self {
            _client: client,
            _github: github,
            _documents: documents,
        }
    }
}

#[tower_lsp::async_trait]
impl Tool for Cargo {}

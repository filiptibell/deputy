use tower_lsp::Client;

use crate::github::*;
use crate::server::*;

use super::*;

#[derive(Debug, Clone)]
pub struct Wally {
    _client: Client,
    _documents: Documents,
    _github: GithubWrapper,
}

impl Wally {
    pub(super) fn new(client: Client, documents: Documents, github: GithubWrapper) -> Self {
        Self {
            _client: client,
            _documents: documents,
            _github: github,
        }
    }
}

#[tower_lsp::async_trait]
impl Tool for Wally {}

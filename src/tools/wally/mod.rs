use tower_lsp::Client;

use crate::github::*;
use crate::server::*;

use super::*;

#[derive(Debug, Clone)]
pub struct Wally {
    _client: Client,
    _github: GithubWrapper,
    _documents: Documents,
}

impl Wally {
    pub(super) fn new(client: Client, github: GithubWrapper, documents: Documents) -> Self {
        Self {
            _client: client,
            _github: github,
            _documents: documents,
        }
    }
}

#[tower_lsp::async_trait]
impl Tool for Wally {}

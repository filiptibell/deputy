use tower_lsp::Client;

use crate::github::*;
use crate::server::*;

use super::*;

#[derive(Debug, Clone)]
pub struct Wally {
    client: Client,
    github: GithubWrapper,
    documents: Documents,
}

impl Wally {
    pub(super) fn new(client: Client, github: GithubWrapper, documents: Documents) -> Self {
        Self {
            client,
            github,
            documents,
        }
    }
}

#[tower_lsp::async_trait]
impl Tool for Wally {}

use tower_lsp::Client;

use crate::github::GithubWrapper;
use crate::server::*;

use super::*;

#[derive(Debug, Clone)]
pub struct Foreman {
    client: Client,
    github: GithubWrapper,
    documents: Documents,
}

impl Foreman {
    pub(super) fn new(client: Client, github: GithubWrapper, documents: Documents) -> Self {
        Self {
            client,
            github,
            documents,
        }
    }
}

#[tower_lsp::async_trait]
impl Tool for Foreman {}

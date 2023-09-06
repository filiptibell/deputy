use std::sync::Arc;

use tower_lsp::Client;

use crate::github::*;
use crate::server::*;
use crate::util::*;

use super::*;

mod dependency_spec;
mod manifest;

use dependency_spec::*;
use manifest::*;

#[derive(Debug, Clone)]
pub struct Wally {
    _client: Client,
    documents: Documents,
    github: GithubWrapper,
}

impl Wally {
    pub(super) fn new(client: Client, documents: Documents, github: GithubWrapper) -> Self {
        Self {
            _client: client,
            documents,
            github,
        }
    }

    async fn get_document(&self, uri: &Url) -> Option<Document> {
        let documents = Arc::clone(&self.documents);
        let documents = documents.lock().await;
        documents.get(uri).cloned()
    }
}

#[tower_lsp::async_trait]
impl Tool for Wally {}

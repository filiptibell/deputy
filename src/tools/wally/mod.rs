use tower_lsp::Client;

use crate::clients::*;
use crate::server::*;

use super::*;

mod dependency_spec;
mod manifest;

use dependency_spec::*;
use manifest::*;

#[derive(Debug, Clone)]
pub struct Wally {
    _client: Client,
    clients: Clients,
    documents: Documents,
}

impl Wally {
    pub(super) fn new(client: Client, clients: Clients, documents: Documents) -> Self {
        Self {
            _client: client,
            clients,
            documents,
        }
    }

    async fn get_document(&self, uri: &Url) -> Option<Document> {
        self.documents.get(uri).map(|r| r.clone())
    }
}

#[tower_lsp::async_trait]
impl Tool for Wally {}

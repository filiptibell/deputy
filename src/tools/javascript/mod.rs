use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::server::*;

use super::*;

mod manifest;

use manifest::*;

#[derive(Debug, Clone)]
pub struct JavaScript {
    _client: Client,
    clients: Clients,
    documents: Documents,
}

impl JavaScript {
    pub(super) fn new(client: Client, clients: Clients, documents: Documents) -> Self {
        Self {
            _client: client,
            clients,
            documents,
        }
    }

    fn get_document(&self, uri: &Url) -> Option<(Document, Manifest)> {
        let document = self.documents.get(uri).map(|r| r.clone())?;
        let manifest = Manifest::parse(document.as_str()).ok()?;
        Some((document, manifest))
    }
}

#[tower_lsp::async_trait]
impl Tool for JavaScript {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        // TODO: Implement this
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
        // TODO: Implement this
        Ok(CompletionResponse::Array(Vec::new()))
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        // TODO: Implement this
        Ok(Vec::new())
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>> {
        // TODO: Implement this
        Ok(Vec::new())
    }
}

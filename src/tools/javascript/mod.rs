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
    _clients: Clients,
    _documents: Documents,
}

impl JavaScript {
    pub(super) fn new(client: Client, clients: Clients, documents: Documents) -> Self {
        Self {
            _client: client,
            _clients: clients,
            _documents: documents,
        }
    }

    fn _get_document(&self, uri: &Url) -> Option<(Document, Manifest)> {
        let document = self._documents.get(uri).map(|r| r.clone())?;
        let manifest = Manifest::parse(document.as_str()).ok()?;
        Some((document, manifest))
    }
}

#[tower_lsp::async_trait]
impl Tool for JavaScript {
    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        // TODO: Implement this
        Ok(None)
    }

    async fn completion(&self, _params: CompletionParams) -> Result<CompletionResponse> {
        // TODO: Implement this
        Ok(CompletionResponse::Array(Vec::new()))
    }

    async fn diagnostics(&self, _params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        // TODO: Implement this
        Ok(Vec::new())
    }

    async fn code_action(&self, _params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>> {
        // TODO: Implement this
        Ok(Vec::new())
    }
}

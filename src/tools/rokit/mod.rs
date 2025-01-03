use futures::future::try_join_all;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tracing::debug;

use crate::parser::query_rokit_toml_dependencies;
use crate::parser::SimpleDependency;
use crate::server::*;
use crate::util::*;

use super::*;

mod diagnostics;
mod hover;

use diagnostics::*;
use hover::*;

#[derive(Debug, Clone)]
pub struct Rokit {
    _client: Client,
    clients: Clients,
    documents: Documents,
}

impl Rokit {
    pub(super) fn new(client: Client, clients: Clients, documents: Documents) -> Self {
        Self {
            _client: client,
            clients,
            documents,
        }
    }

    fn get_document(&self, uri: &Url) -> Option<Document> {
        if uri
            .file_name()
            .as_deref()
            .is_some_and(|f| f.eq_ignore_ascii_case("rokit.toml"))
        {
            self.documents.get(uri).map(|r| r.clone())
        } else {
            None
        }
    }
}

#[tower_lsp::async_trait]
impl Tool for Rokit {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(doc) = self.get_document(&uri) else {
            return Ok(None);
        };

        // Find the dependency that is hovered over
        let tools = query_rokit_toml_dependencies(doc.inner());
        let Some(found) = SimpleDependency::find_at_pos(&tools, pos) else {
            return Ok(None);
        };

        // Fetch some extra info and return the hover
        debug!("Hovering: {found:?}");
        get_rokit_hover(&self.clients, &doc, found).await
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        let uri = params.text_document.uri;
        let Some(doc) = self.get_document(&uri) else {
            return Ok(Vec::new());
        };

        // Find all tools
        let tools = query_rokit_toml_dependencies(doc.inner());
        if tools.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch all diagnostics concurrently
        debug!("Fetching rokit diagnostics for tools");
        let results = try_join_all(
            tools
                .iter()
                .map(|tool| get_rokit_diagnostics(&self.clients, &doc, tool)),
        )
        .await?;

        Ok(results.into_iter().flatten().collect())
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>> {
        let mut actions = Vec::new();
        for diag in params.context.diagnostics {
            if let Some(Ok(action)) = diag
                .data
                .as_ref()
                .map(ResolveContext::<CodeActionMetadata>::try_from)
            {
                actions.push(action.into_inner().into_code_action(diag.clone()))
            }
        }
        Ok(actions)
    }
}

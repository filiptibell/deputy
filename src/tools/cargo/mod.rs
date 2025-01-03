use futures::future::try_join_all;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tracing::debug;

use crate::parser::query_cargo_toml_dependencies;
use crate::parser::Dependency;
use crate::server::*;
use crate::util::*;

use super::*;

mod completion;
mod constants;
mod diagnostics;
mod hover;

use completion::*;
use diagnostics::*;
use hover::*;

#[derive(Debug, Clone)]
pub struct Cargo {
    _client: Client,
    clients: Clients,
    documents: Documents,
}

impl Cargo {
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
            .is_some_and(|f| f.eq_ignore_ascii_case("Cargo.toml"))
        {
            self.documents.get(uri).map(|r| r.clone())
        } else {
            None
        }
    }
}

#[tower_lsp::async_trait]
impl Tool for Cargo {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(doc) = self.get_document(&uri) else {
            return Ok(None);
        };

        // Find the dependency that is hovered over
        let dependencies = query_cargo_toml_dependencies(doc.inner());
        let Some(found) = Dependency::find_at_pos(&dependencies, pos) else {
            return Ok(None);
        };

        // Fetch some extra info and return the hover
        debug!("Hovering: {found:?}");
        get_cargo_hover(&self.clients, &doc, found).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let Some(doc) = self.get_document(&uri) else {
            return Ok(CompletionResponse::Array(Vec::new()));
        };

        // Find the dependency that is being completed
        let dependencies = query_cargo_toml_dependencies(doc.inner());
        let Some(found) = Dependency::find_at_pos(&dependencies, pos) else {
            return Ok(CompletionResponse::Array(Vec::new()));
        };

        // Check what we're completing - name or version
        if found.name().contains(pos) {
            debug!("Completing name: {found:?}");
            get_cargo_completions_name(&self.clients, &doc, found).await
        } else if found.spec().is_some_and(|s| s.contains(pos)) {
            debug!("Completing version: {found:?}");
            get_cargo_completions_version(&self.clients, &doc, found).await
        } else {
            Ok(CompletionResponse::Array(Vec::new()))
        }
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        let uri = params.text_document.uri;
        let Some(doc) = self.get_document(&uri) else {
            return Ok(Vec::new());
        };

        // Find all dependencies
        let dependencies = query_cargo_toml_dependencies(doc.inner());
        if dependencies.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch all diagnostics concurrently
        debug!("Fetching cargo diagnostics for dependencies");
        let results = try_join_all(
            dependencies
                .iter()
                .map(|dep| get_cargo_diagnostics(&self.clients, &doc, dep)),
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

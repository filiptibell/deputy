use futures::future::try_join_all;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tracing::debug;

use crate::parser::query_wally_toml_dependencies;
use crate::parser::SimpleDependency;
use crate::server::*;
use crate::util::*;

use super::*;

mod diagnostics;
mod hover;

use diagnostics::*;
use hover::*;

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

    fn get_document(&self, uri: &Url) -> Option<Document> {
        if uri
            .file_name()
            .as_deref()
            .is_some_and(|f| f.eq_ignore_ascii_case("wally.toml"))
        {
            self.documents.get(uri).map(|r| r.clone())
        } else {
            None
        }
    }
}

#[tower_lsp::async_trait]
impl Tool for Wally {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(doc) = self.get_document(&uri) else {
            return Ok(None);
        };

        let index_url = extract_wally_index_url(doc.as_str());

        // Find the dependency that is hovered over
        let dependencies = query_wally_toml_dependencies(doc.inner());
        let Some(found) = SimpleDependency::find_at_pos(&dependencies, pos) else {
            return Ok(None);
        };

        // Fetch some extra info and return the hover
        debug!("Hovering: {found:?}");
        get_wally_hover(&self.clients, &doc, index_url, found).await
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        let uri = params.text_document.uri;
        let Some(doc) = self.get_document(&uri) else {
            return Ok(Vec::new());
        };

        let index_url = extract_wally_index_url(doc.as_str());

        // Find all dependencies
        let dependencies = query_wally_toml_dependencies(doc.inner());
        if dependencies.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch all diagnostics concurrently
        debug!("Fetching wally diagnostics for dependencies");
        let results = try_join_all(
            dependencies
                .iter()
                .map(|tool| get_wally_diagnostics(&self.clients, &doc, index_url, tool)),
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

fn extract_wally_index_url(doc_contents: &str) -> &str {
    doc_contents
        .lines()
        .find_map(|line| {
            line.split_once('=')
                .filter(|(key, _)| key.trim() == "registry")
                .map(|(_, value)| {
                    value
                        .trim()
                        .trim_start_matches(['\'', '\"'])
                        .trim_end_matches(['\'', '\"'])
                        .trim_end_matches(".git")
                })
        })
        .unwrap_or("https://github.com/UpliftGames/wally-index")
}

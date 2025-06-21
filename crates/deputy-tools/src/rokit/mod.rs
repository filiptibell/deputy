use futures::future::try_join_all;
use tracing::debug;

use async_language_server::{
    lsp_types::{CompletionResponse, Diagnostic, DocumentDiagnosticParams, Hover, Position},
    server::{Document, ServerResult},
    tree_sitter::Node,
};

use deputy_clients::Clients;
use deputy_parser::rokit;

mod completion;
mod constants;
mod diagnostics;
mod hover;

use completion::get_rokit_completions;
use diagnostics::get_rokit_diagnostics;
use hover::get_rokit_hover;

#[derive(Debug, Clone)]
pub struct Rokit {
    clients: Clients,
}

impl Rokit {
    pub(super) fn new(clients: Clients) -> Self {
        Self { clients }
    }

    pub(super) async fn hover(
        &self,
        doc: &Document,
        pos: Position,
        _node: Node<'_>,
    ) -> ServerResult<Option<Hover>> {
        let Some(dep) = rokit::find_dependency_at(doc, pos) else {
            return Ok(None);
        };

        debug!("Hovering: {dep:?}");

        get_rokit_hover(&self.clients, doc, dep).await
    }

    pub(super) async fn completion(
        &self,
        doc: &Document,
        pos: Position,
        _node: Node<'_>,
    ) -> ServerResult<Option<CompletionResponse>> {
        let Some(dep) = rokit::find_dependency_at(doc, pos) else {
            return Ok(None);
        };

        debug!("Fetching completions: {dep:?}");

        get_rokit_completions(&self.clients, doc, pos, dep).await
    }

    pub(super) async fn diagnostics(
        &self,
        doc: &Document,
        _params: DocumentDiagnosticParams,
    ) -> ServerResult<Vec<Diagnostic>> {
        // Find all dependencies
        let dependencies = rokit::find_all_dependencies(doc);
        if dependencies.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch all diagnostics concurrently
        debug!("Fetching rokit diagnostics for dependencies");
        let results = try_join_all(
            dependencies
                .into_iter()
                .map(|node| get_rokit_diagnostics(&self.clients, doc, node)),
        )
        .await?;

        Ok(results.into_iter().flatten().collect())
    }
}

use async_language_server::{
    lsp_types::{CompletionResponse, Diagnostic, DocumentDiagnosticParams, Hover, Position},
    server::{Document, ServerResult},
    tree_sitter::Node,
};
use futures::future::try_join_all;
use tracing::debug;

use deputy_clients::Clients;
use deputy_parser::pyproject;

mod completion;
mod constants;
mod diagnostics;
mod hover;

use self::completion::get_pyproject_completions;
use self::diagnostics::get_pyproject_diagnostics;
use self::hover::get_pyproject_hover;

#[derive(Debug, Clone)]
pub struct PyProject {
    clients: Clients,
}

impl PyProject {
    pub(super) fn new(clients: Clients) -> Self {
        Self { clients }
    }

    pub(super) async fn hover(
        &self,
        doc: &Document,
        pos: Position,
        _node: Node<'_>,
    ) -> ServerResult<Option<Hover>> {
        let Some(dep) = pyproject::find_dependency_at(doc, pos) else {
            return Ok(None);
        };

        debug!("Hovering: {dep:?}");

        get_pyproject_hover(&self.clients, doc, dep).await
    }

    pub(super) async fn completion(
        &self,
        doc: &Document,
        pos: Position,
        _node: Node<'_>,
    ) -> ServerResult<Option<CompletionResponse>> {
        let Some(dep) = pyproject::find_dependency_at(doc, pos) else {
            return Ok(None);
        };

        debug!("Fetching completions: {dep:?}");

        get_pyproject_completions(&self.clients, doc, pos, dep).await
    }

    pub(super) async fn diagnostics(
        &self,
        doc: &Document,
        _params: DocumentDiagnosticParams,
    ) -> ServerResult<Vec<Diagnostic>> {
        // Find all dependencies
        let dependencies = pyproject::find_all_dependencies(doc);
        if dependencies.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch all diagnostics concurrently
        debug!("Fetching pyproject diagnostics for dependencies");
        let results = try_join_all(
            dependencies
                .into_iter()
                .map(|node| get_pyproject_diagnostics(&self.clients, doc, node)),
        )
        .await?;

        Ok(results.into_iter().flatten().collect())
    }
}

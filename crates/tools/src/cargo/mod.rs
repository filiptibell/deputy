use futures::future::try_join_all;
use tracing::debug;

use async_language_server::{
    lsp_types::{CompletionResponse, Diagnostic, DocumentDiagnosticParams, Hover, Position},
    server::{Document, ServerResult},
    tree_sitter::Node,
};

use clients::Clients;
use parser::cargo;

mod completion;
mod constants;
mod diagnostics;
mod hover;
mod util;

use completion::get_cargo_completions;
use diagnostics::get_cargo_diagnostics;
use hover::get_cargo_hover;

#[derive(Debug, Clone)]
pub struct Cargo {
    clients: Clients,
}

impl Cargo {
    pub(super) fn new(clients: Clients) -> Self {
        Self { clients }
    }

    pub(super) async fn hover(
        &self,
        doc: &Document,
        pos: Position,
        _node: Node<'_>,
    ) -> ServerResult<Option<Hover>> {
        let Some(dep) = cargo::find_dependency_at(doc, pos) else {
            return Ok(None);
        };

        debug!("Hovering: {dep:?}");

        get_cargo_hover(&self.clients, doc, dep).await
    }

    pub(super) async fn completion(
        &self,
        doc: &Document,
        pos: Position,
        _node: Node<'_>,
    ) -> ServerResult<Option<CompletionResponse>> {
        let Some(dep) = cargo::find_dependency_at(doc, pos) else {
            return Ok(None);
        };

        debug!("Fetching completions: {dep:?}");

        get_cargo_completions(&self.clients, doc, pos, dep).await
    }

    pub(super) async fn diagnostics(
        &self,
        doc: &Document,
        _params: DocumentDiagnosticParams,
    ) -> ServerResult<Vec<Diagnostic>> {
        // Find all dependencies
        let dependencies = cargo::find_all_dependencies(doc);
        if dependencies.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch all diagnostics concurrently
        debug!("Fetching cargo diagnostics for dependencies");
        let results = try_join_all(
            dependencies
                .into_iter()
                .map(|node| get_cargo_diagnostics(&self.clients, doc, node)),
        )
        .await?;

        Ok(results.into_iter().flatten().collect())
    }
}

use futures::future::try_join_all;
use tracing::debug;

use async_language_server::{
    lsp_types::{CompletionResponse, Diagnostic, DocumentDiagnosticParams, Hover},
    server::{Document, ServerResult},
};

use crate::parser::query_package_json_dependencies;
use crate::parser::Dependency;

use super::*;

mod completion;
mod constants;
mod diagnostics;
mod hover;

use completion::*;
use diagnostics::*;
use hover::*;

#[derive(Debug, Clone)]
pub struct Npm {
    clients: Clients,
}

impl Npm {
    pub(super) fn new(clients: Clients) -> Self {
        Self { clients }
    }

    pub(super) async fn hover(
        &self,
        doc: &Document,
        pos: Position,
        _node: Node<'_>,
    ) -> ServerResult<Option<Hover>> {
        // Find the dependency that is hovered over
        let dependencies = query_package_json_dependencies(doc);
        let Some(found) = Dependency::find_at_pos(&dependencies, pos) else {
            return Ok(None);
        };

        // Fetch some extra info and return the hover
        debug!("Hovering: {found:?}");
        get_npm_hover(&self.clients, doc, found).await
    }

    pub(super) async fn completion(
        &self,
        doc: &Document,
        pos: Position,
        _node: Node<'_>,
    ) -> ServerResult<Option<CompletionResponse>> {
        // Find the dependency that is being completed
        let dependencies = query_package_json_dependencies(doc);
        let Some(found) = Dependency::find_at_pos(&dependencies, pos) else {
            return Ok(None);
        };

        // Check what we're completing - name or version
        if found.name().contains(pos) {
            debug!("Completing name: {found:?}");
            return get_npm_completions_name(&self.clients, doc, found).await;
        } else if let Some(s) = found.spec().filter(|s| s.contains(pos)) {
            if s.contents.version.as_ref().is_some_and(|v| v.contains(pos)) {
                debug!("Completing version: {found:?}");
                return get_npm_completions_version(&self.clients, doc, found).await;
            }
        }

        Ok(None)
    }

    pub(super) async fn diagnostics(
        &self,
        doc: &Document,
        _params: DocumentDiagnosticParams,
    ) -> ServerResult<Vec<Diagnostic>> {
        // Find all dependencies
        let dependencies = query_package_json_dependencies(doc);
        if dependencies.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch all diagnostics concurrently
        debug!("Fetching npm diagnostics for dependencies");
        let results = try_join_all(
            dependencies
                .iter()
                .map(|dep| get_npm_diagnostics(&self.clients, doc, dep)),
        )
        .await?;

        Ok(results.into_iter().flatten().collect())
    }
}

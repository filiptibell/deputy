use futures::future::try_join_all;
use tracing::debug;

use async_language_server::{
    lsp_types::{CompletionResponse, Diagnostic, DocumentDiagnosticParams, Hover},
    server::{Document, ServerResult},
};

use crate::parser::query_rokit_toml_dependencies;
use crate::parser::SimpleDependency;

use super::*;

mod completion;
mod constants;
mod diagnostics;
mod hover;

use completion::*;
use diagnostics::*;
use hover::*;

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
        // Find the dependency that is hovered over
        let dependencies = query_rokit_toml_dependencies(doc);
        let Some(found) = SimpleDependency::find_at_pos(&dependencies, pos) else {
            return Ok(None);
        };

        // Fetch some extra info and return the hover
        debug!("Hovering: {found:?}");
        get_rokit_hover(&self.clients, doc, found).await
    }

    pub(super) async fn completion(
        &self,
        doc: &Document,
        pos: Position,
        _node: Node<'_>,
    ) -> ServerResult<Option<CompletionResponse>> {
        // Find the dependency that is being completed
        let dependencies = query_rokit_toml_dependencies(doc);
        let Some(found) = SimpleDependency::find_at_pos(&dependencies, pos) else {
            return Ok(None);
        };

        // Check what we're completing - author, name, or version
        let parsed = found.parsed_spec();
        if parsed.version.as_ref().is_some_and(|v| v.contains(pos)) {
            debug!("Completing version: {found:?}");
            return get_rokit_completions_spec_version(&self.clients, doc, found).await;
        } else if parsed.name.is_some_and(|n| n.contains(pos)) {
            debug!("Completing name: {found:?}");
            return get_rokit_completions_spec_name(&self.clients, doc, found).await;
        } else if parsed.author.contains(pos)
            || (parsed.author.unquoted().is_empty() && found.spec.contains(pos))
        {
            debug!("Completing author: {found:?}");
            return get_rokit_completions_spec_author(&self.clients, doc, found).await;
        }

        Ok(None)
    }

    pub(super) async fn diagnostics(
        &self,
        doc: &Document,
        _params: DocumentDiagnosticParams,
    ) -> ServerResult<Vec<Diagnostic>> {
        // Find all dependencies
        let dependencies = query_rokit_toml_dependencies(doc);
        if dependencies.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch all diagnostics concurrently
        debug!("Fetching rokit diagnostics for dependencies");
        let results = try_join_all(
            dependencies
                .iter()
                .map(|tool| get_rokit_diagnostics(&self.clients, doc, tool)),
        )
        .await?;

        Ok(results.into_iter().flatten().collect())
    }
}

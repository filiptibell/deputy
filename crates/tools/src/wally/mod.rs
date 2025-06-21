use std::io::{BufRead, BufReader};

use futures::future::try_join_all;
use tracing::debug;

use async_language_server::{
    lsp_types::{CompletionResponse, Diagnostic, DocumentDiagnosticParams, Hover, Position},
    server::{Document, ServerResult},
    tree_sitter::Node,
};

use clients::Clients;
use parser::wally;

mod completion;
mod constants;
mod diagnostics;
mod hover;

use completion::get_wally_completions;
use constants::WALLY_DEFAULT_REGISTRY;
use diagnostics::get_wally_diagnostics;
use hover::get_wally_hover;

#[derive(Debug, Clone)]
pub struct Wally {
    clients: Clients,
}

impl Wally {
    pub(super) fn new(clients: Clients) -> Self {
        Self { clients }
    }

    pub(super) async fn hover(
        &self,
        doc: &Document,
        pos: Position,
        _node: Node<'_>,
    ) -> ServerResult<Option<Hover>> {
        let Some(dep) = wally::find_dependency_at(doc, pos) else {
            return Ok(None);
        };

        let index_url = extract_wally_index_url(doc);

        debug!("Hovering: {dep:?}");

        get_wally_hover(&self.clients, doc, index_url.as_str(), dep).await
    }

    pub(super) async fn completion(
        &self,
        doc: &Document,
        pos: Position,
        _node: Node<'_>,
    ) -> ServerResult<Option<CompletionResponse>> {
        let Some(dep) = wally::find_dependency_at(doc, pos) else {
            return Ok(None);
        };

        let index_url = extract_wally_index_url(doc);

        debug!("Fetching completions: {dep:?}");

        get_wally_completions(&self.clients, doc, pos, index_url.as_str(), dep).await
    }

    pub(super) async fn diagnostics(
        &self,
        doc: &Document,
        _params: DocumentDiagnosticParams,
    ) -> ServerResult<Vec<Diagnostic>> {
        // Find all dependencies
        let dependencies = wally::find_all_dependencies(doc);
        if dependencies.is_empty() {
            return Ok(Vec::new());
        }

        let index_url = extract_wally_index_url(doc);

        // Fetch all diagnostics concurrently
        debug!("Fetching wally diagnostics for dependencies");
        let results = try_join_all(
            dependencies
                .into_iter()
                .map(|node| get_wally_diagnostics(&self.clients, doc, index_url.as_str(), node)),
        )
        .await?;

        Ok(results.into_iter().flatten().collect())
    }
}

fn extract_wally_index_url(doc: &Document) -> String {
    let mut reader = BufReader::new(doc.text_reader());

    let mut line = String::new();
    while reader.read_line(&mut line).is_ok() {
        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "registry" {
                return value
                    .trim()
                    .trim_start_matches(['\'', '\"'])
                    .trim_end_matches(['\'', '\"'])
                    .trim_end_matches(".git")
                    .to_string();
            }
        }
        line.clear();
    }

    WALLY_DEFAULT_REGISTRY.to_string()
}

use std::collections::HashMap;
use std::sync::Arc;

use tracing::trace;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::server::*;

use super::*;

mod manifest;
use manifest::*;

#[derive(Debug, Clone)]
pub struct Cargo {
    _client: Client,
    documents: Documents,
}

impl Cargo {
    pub(super) fn new(client: Client, documents: Documents) -> Self {
        Self {
            _client: client,
            documents,
        }
    }

    async fn get_document(&self, uri: &Url) -> Option<Document> {
        let documents = Arc::clone(&self.documents);
        let documents = documents.lock().await;
        documents.get(uri).cloned()
    }
}

#[tower_lsp::async_trait]
impl Tool for Cargo {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let document = match self.get_document(&uri).await {
            None => return Ok(None),
            Some(d) => d,
        };
        let manifest = match Manifest::parse(document.as_str()) {
            Err(_) => return Ok(None),
            Ok(m) => m,
        };

        let offset = document.lsp_position_to_offset(pos);
        let try_find = |deps: &HashMap<String, ManifestDependency>| {
            deps.iter().find_map(|(_, dep)| {
                let span = dep.span();
                if offset >= span.start && offset <= span.end {
                    Some((document.lsp_range_from_range(span.clone()), dep.to_string()))
                } else {
                    None
                }
            })
        };

        let found = try_find(&manifest.dependencies)
            .or_else(|| try_find(&manifest.dev_dependencies))
            .or_else(|| try_find(&manifest.build_dependencies));
        let (found_range, found_dep) = match found {
            Some((range, dep)) => (range, dep),
            _ => return Ok(None),
        };

        trace!("Hovering: {found_dep}");

        let mut lines = Vec::new();
        lines.push(format!("## {}", found_dep));

        Ok(Some(Hover {
            range: Some(found_range),
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: lines.join("\n"),
            }),
        }))
    }
}

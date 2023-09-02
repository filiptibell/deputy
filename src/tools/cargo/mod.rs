use std::collections::HashMap;
use std::sync::Arc;

use tracing::trace;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::crates::*;
use crate::server::*;

use super::*;

mod manifest;
use manifest::*;

#[derive(Debug, Clone)]
pub struct Cargo {
    _client: Client,
    documents: Documents,
    crates: CratesWrapper,
}

impl Cargo {
    pub(super) fn new(client: Client, documents: Documents, crates: CratesWrapper) -> Self {
        Self {
            _client: client,
            documents,
            crates,
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
            deps.iter().find_map(|(key, dep)| {
                let span = dep.span();
                if offset >= span.start && offset <= span.end {
                    Some((
                        document.lsp_range_from_range(span.clone()),
                        key.to_string(),
                        dep.to_string(),
                    ))
                } else {
                    None
                }
            })
        };

        let found = try_find(&manifest.dependencies)
            .or_else(|| try_find(&manifest.dev_dependencies))
            .or_else(|| try_find(&manifest.build_dependencies));
        let (found_range, found_key, found_ver) = match found {
            Some((range, key, dep)) => (range, key, dep),
            _ => return Ok(None),
        };

        trace!("Hovering: {found_key} version {found_ver}");

        let metadatas = match self.crates.get_index_metadatas(&found_key).await {
            Err(_) => return Ok(None),
            Ok(m) => m,
        };
        let meta_latest = match metadatas.last() {
            None => return Ok(None),
            Some(m) => m,
        };

        let mut lines = Vec::new();
        lines.push(format!("## {}", meta_latest.name));
        lines.push(format!("Version **{}**", meta_latest.version));

        trace!("Fetching crate data from crates.io");
        if let Ok(crate_data) = self
            .crates
            .get_crate_data(&found_key)
            .await
            .map(|c| c.inner)
        {
            lines.push(String::new());
            lines.push(crate_data.description.to_string());
            let mut docs = crate_data.links.documentation.as_deref();
            let mut page = crate_data.links.homepage.as_deref();
            let repo = crate_data.links.repository.as_deref();
            if docs.is_some() || page.is_some() || repo.is_some() {
                if page == repo {
                    page = None;
                }
                if docs == repo {
                    docs = None;
                }
                lines.push(String::new());
                lines.push(String::from("### Links"));
                if let Some(docs) = docs {
                    lines.push(format!("- [Documentation]({docs})"));
                }
                if let Some(repo) = repo {
                    lines.push(format!("- [Repository]({repo})"));
                }
                if let Some(page) = page {
                    lines.push(format!("- [Homepage]({page})"));
                }
                lines.push(String::new());
            }
        }

        Ok(Some(Hover {
            range: Some(found_range),
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: lines.join("\n"),
            }),
        }))
    }
}

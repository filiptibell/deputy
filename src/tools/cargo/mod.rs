use std::collections::HashMap;
use std::sync::Arc;

use tracing::trace;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tracing::warn;

use crate::crates::*;
use crate::server::*;
use crate::util::*;

use super::*;

mod lockfile;
mod manifest;

use lockfile::*;
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

    async fn get_documents(&self, uri: &Url) -> Option<(Document, Document)> {
        if matches!(
            uri.file_name().as_deref(),
            Some("Cargo.toml" | "cargo.toml")
        ) {
            let documents = Arc::clone(&self.documents);
            let documents = documents.lock().await;

            let manifest = documents.get(uri).cloned();
            let lockfile = documents
                .get(&uri.with_file_name("Cargo.lock").unwrap())
                .cloned();

            if lockfile.is_none() {
                warn!("Cargo.lock missing for manifest at '{uri}'")
            }

            match (manifest, lockfile) {
                (Some(m), Some(l)) => Some((m, l)),
                _ => None,
            }
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

        let (document, lockdoc) = match self.get_documents(&uri).await {
            None => return Ok(None),
            Some(d) => d,
        };
        let (manifest, lockfile) = match (
            document.as_str().parse::<Manifest>(),
            lockdoc.as_str().parse::<Lockfile>(),
        ) {
            (Ok(m), Ok(l)) => (m, l),
            _ => return Ok(None),
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

        let locked = match lockfile
            .packages
            .iter()
            .find(|package| package.name.eq_ignore_ascii_case(&found_key))
        {
            Some(package) => package,
            None => return Ok(None),
        };

        let mut lines = Vec::new();
        lines.push(format!("## {}", locked.name));
        lines.push(format!("Version **{}**", locked.version));

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

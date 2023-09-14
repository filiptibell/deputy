use std::collections::HashMap;

use futures::future::join_all;
use tracing::trace;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::server::*;
use crate::util::*;

use super::*;

mod completion;
mod diagnostics;
mod lockfile;
mod manifest;

use completion::*;
use diagnostics::*;
use lockfile::*;
use manifest::*;

#[derive(Debug, Clone)]
pub struct Cargo {
    _client: Client,
    clients: Clients,
    documents: Documents,
}

impl Cargo {
    pub(super) fn new(client: Client, clients: Clients, documents: Documents) -> Self {
        Self {
            _client: client,
            clients,
            documents,
        }
    }

    fn get_documents(&self, uri: &Url) -> Option<(Document, Document, Manifest, Lockfile)> {
        if !matches!(
            uri.file_name().as_deref(),
            Some("Cargo.toml" | "cargo.toml")
        ) {
            return None;
        }

        let doc_manifest = self.documents.get(uri).map(|r| r.clone())?;
        let doc_lockfile = self
            .documents
            .get(&uri.with_file_name("Cargo.lock").unwrap())
            .map(|r| r.clone())?;

        let manifest = doc_manifest.as_str().parse::<Manifest>().ok()?;
        let lockfile = doc_lockfile.as_str().parse::<Lockfile>().ok()?;

        Some((doc_manifest, doc_lockfile, manifest, lockfile))
    }
}

#[tower_lsp::async_trait]
impl Tool for Cargo {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let (document, _, manifest, lockfile) = match self.get_documents(&uri) {
            None => return Ok(None),
            Some(d) => d,
        };

        let offset = document.lsp_position_to_offset(pos);
        let try_find = |deps: &HashMap<String, ManifestDependency>| {
            deps.iter().find_map(|(key, dep)| {
                let span_name = dep.name_span();
                let span_version = dep.version_span();

                let hovered_span = if offset >= span_version.start && offset <= span_version.end {
                    Some(span_version)
                } else if offset >= span_name.start && offset <= span_name.end {
                    Some(span_name)
                } else {
                    None
                };

                hovered_span.map(|span| {
                    (
                        document.lsp_range_from_span(span.clone()),
                        key.to_string(),
                        dep.version_source().to_string(),
                    )
                })
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
            .clients
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

    async fn completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let (document, _, manifest, _) = match self.get_documents(&uri) {
            None => return Ok(CompletionResponse::Array(Vec::new())),
            Some(d) => d,
        };

        let offset = document.lsp_position_to_offset(pos);
        let try_find = |deps: &HashMap<String, ManifestDependency>| {
            deps.iter().find_map(|(_, dep)| {
                let span = dep.version_span();
                if offset >= span.start && offset <= span.end {
                    Some((span.clone(), dep.clone()))
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
            _ => return Ok(CompletionResponse::Array(Vec::new())),
        };

        let range_before = document.lsp_range_to_span(Range {
            start: document.lsp_position_from_offset(found_range.start + 1),
            end: pos,
        });

        let slice_before = &document.as_str()[range_before.clone()];
        get_package_completions(
            &self.clients,
            &document,
            range_before,
            found_dep.name_text(),
            slice_before,
        )
        .await
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        let uri = params.text_document.uri;

        let (document, _, manifest, _) = match self.get_documents(&uri) {
            None => return Ok(Vec::new()),
            Some(d) => d,
        };

        let deps = (manifest.dependencies.values())
            .chain(manifest.dev_dependencies.values())
            .chain(manifest.build_dependencies.values())
            .map(|dep| {
                let range_name = document.lsp_range_from_span(dep.name_span().clone());
                let range_version = document.lsp_range_from_span(dep.version_span().clone());
                (dep.clone(), range_name, range_version)
            })
            .collect::<Vec<_>>();

        let mut all_diagnostics = Vec::new();
        let mut fut_diagnostics = Vec::new();
        for (tool, range_name, range_version) in &deps {
            fut_diagnostics.push(diagnose_dependency(
                &self.clients,
                &uri,
                tool,
                range_name,
                range_version,
            ));
        }

        for diag in join_all(fut_diagnostics).await.into_iter().flatten() {
            all_diagnostics.push(diag);
        }

        Ok(all_diagnostics)
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

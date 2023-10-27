use std::collections::HashMap;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tracing::trace;

use crate::server::*;
use crate::util::*;

use super::*;

mod completion;
mod constants;
mod lockfile;
mod manifest;

use completion::*;
use lockfile::*;
use manifest::*;

#[derive(Debug, Clone)]
pub struct JavaScript {
    _client: Client,
    clients: Clients,
    documents: Documents,
}

impl JavaScript {
    pub(super) fn new(client: Client, clients: Clients, documents: Documents) -> Self {
        Self {
            _client: client,
            clients,
            documents,
        }
    }

    fn get_documents(&self, uri: &Url) -> Option<(Document, Document, Manifest, Lockfile)> {
        if !matches!(uri.file_name().as_deref(), Some("package.json")) {
            return None;
        }

        let doc_manifest = self.documents.get(uri).map(|r| r.clone())?;
        let doc_lockfile = self
            .documents
            .get(&uri.with_file_name("package-lock.json").unwrap())
            .map(|r| r.clone())?;

        let manifest = doc_manifest.as_str().parse::<Manifest>().ok()?;
        let lockfile = doc_lockfile.as_str().parse::<Lockfile>().ok()?;

        Some((doc_manifest, doc_lockfile, manifest, lockfile))
    }
}

#[tower_lsp::async_trait]
impl Tool for JavaScript {
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
            .or_else(|| try_find(&manifest.build_dependencies))
            .or_else(|| try_find(&manifest.optional_dependencies));
        let (found_range, found_key, found_ver) = match found {
            Some((range, key, dep)) => (range, key, dep),
            _ => return Ok(None),
        };

        trace!("Hovering: {found_key} version {found_ver}");

        let (locked_name, locked_package) = match lockfile.packages.iter().find_map(|(key, val)| {
            let name = key.trim_start_matches("node_modules/");
            if name.eq_ignore_ascii_case(&found_key) {
                Some((name, val))
            } else {
                None
            }
        }) {
            Some(package) => package,
            None => return Ok(None),
        };

        let mut lines = Vec::new();
        lines.push(format!("## {}", locked_name));
        lines.push(format!("Version **{}**", locked_package.version));

        trace!("Fetching package data from npm");
        if let Ok(package_data) = self
            .clients
            .npm
            .get_registry_metadata(&found_key)
            .await
            .map(|c| c.current_version)
        {
            if let Some(desc) = package_data.description {
                lines.push(String::new());
                lines.push(desc.to_string());
            }

            if package_data.homepage.is_some() || package_data.repository.is_some() {
                let page = package_data.homepage.as_deref();
                let repo = package_data.repository.and_then(|r| r.url()).map(|p| {
                    p.trim_start_matches("git+")
                        .trim_end_matches(".git")
                        .to_string()
                });

                lines.push(String::new());
                lines.push(String::from("### Links"));
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

        // TODO: Completion for package names

        let found = try_find(&manifest.dependencies)
            .or_else(|| try_find(&manifest.dev_dependencies))
            .or_else(|| try_find(&manifest.build_dependencies))
            .or_else(|| try_find(&manifest.optional_dependencies));
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

    async fn diagnostics(&self, _params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        // TODO: Implement this
        Ok(Vec::new())
    }

    async fn code_action(&self, _params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>> {
        // TODO: Implement this
        Ok(Vec::new())
    }
}

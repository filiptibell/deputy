use std::collections::HashMap;

use futures::future::join_all;
use semver::Version;
use tower_lsp::Client;
use tracing::trace;

use crate::server::*;

use super::*;

mod completion;
mod constants;
mod diagnostics;
mod manifest;

use completion::*;
use constants::*;
use diagnostics::*;
use manifest::*;

#[derive(Debug, Clone)]
pub struct Wally {
    _client: Client,
    clients: Clients,
    documents: Documents,
}

impl Wally {
    pub(super) fn new(client: Client, clients: Clients, documents: Documents) -> Self {
        Self {
            _client: client,
            clients,
            documents,
        }
    }

    fn get_documents(&self, uri: &Url) -> Option<(Document, Manifest, String)> {
        let document = self.documents.get(uri).map(|r| r.clone())?;
        let manifest = Manifest::parse(document.as_str()).ok()?;

        let primary_index_url = match &manifest.metadata {
            None => return None,
            Some(m) => m.package.registry.clone(),
        };

        Some((document, manifest, primary_index_url))
    }
}

#[tower_lsp::async_trait]
impl Tool for Wally {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let (document, manifest, index_url) = match self.get_documents(&uri) {
            None => return Ok(None),
            Some(d) => d,
        };

        let offset = document.lsp_position_to_offset(pos);
        let try_find = |deps: &HashMap<String, ManifestDependency>| {
            deps.iter().find_map(|(_, dep)| {
                let span = dep.span();
                if offset >= span.start && offset <= span.end {
                    Some((document.lsp_range_from_span(span.clone()), dep.clone()))
                } else {
                    None
                }
            })
        };

        let found = try_find(&manifest.dependencies)
            .or_else(|| try_find(&manifest.dev_dependencies))
            .or_else(|| try_find(&manifest.server_dependencies));

        let (found_range, found_ver) = match found {
            Some((range, dep)) => (range, dep),
            _ => return Ok(None),
        };
        let found_spec = match found_ver.spec() {
            Err(_) => return Ok(None),
            Ok(s) => s,
        };

        trace!("Hovering: {found_spec:?}");

        let metadatas = self
            .clients
            .wally
            .get_index_metadatas(&index_url, &found_spec.author, &found_spec.name)
            .await;
        let metadatas = match metadatas {
            Err(_) => return Ok(None),
            Ok(m) => m,
        };

        let mut lines = Vec::new();
        lines.push(format!("## {}", found_spec.name));

        let exact_match = metadatas
            .iter()
            .find(|m| found_spec.version == m.package.version);
        let version_match = metadatas.iter().find(|m| {
            Version::parse(&m.package.version)
                .map(|version| found_spec.version_req.matches(&version))
                .unwrap_or_default()
        });
        if let Some(best_match) = exact_match.or(version_match) {
            lines.push(format!(
                "By **{}** - **{}**",
                format_authors(
                    &found_spec.author,
                    &best_match
                        .package
                        .authors
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                ),
                best_match.package.version
            ));
            if let Some(desc) = &best_match.package.description {
                lines.push(String::new());
                lines.push(desc.to_string());
            }
            if is_default_index(&best_match.package.registry) {
                lines.push(String::new());
                lines.push(String::from("### Links"));
                lines.push(format!(
                    "- [Wally]({})",
                    get_default_frontend_link(
                        &best_match.package.name,
                        Some(&best_match.package.version)
                    )
                ));
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

        let (document, manifest, index_url) = match self.get_documents(&uri) {
            None => return Ok(CompletionResponse::Array(Vec::new())),
            Some(d) => d,
        };

        let offset = document.lsp_position_to_offset(pos);
        let try_find = |deps: &HashMap<String, ManifestDependency>| {
            deps.iter().find_map(|(_, dep)| {
                let span = dep.span();
                if offset >= span.start && offset <= span.end {
                    Some((span.clone(), dep.clone()))
                } else {
                    None
                }
            })
        };

        let found = try_find(&manifest.dependencies)
            .or_else(|| try_find(&manifest.dev_dependencies))
            .or_else(|| try_find(&manifest.server_dependencies));
        let (found_range, _) = match found {
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
            &index_url,
            range_before,
            slice_before,
        )
        .await
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        let uri = params.text_document.uri;

        let (document, manifest, index_url) = match self.get_documents(&uri) {
            None => return Ok(Vec::new()),
            Some(d) => d,
        };

        let deps = (manifest.dependencies.values())
            .chain(manifest.dev_dependencies.values())
            .chain(manifest.server_dependencies.values())
            .map(|dep| {
                let range = document.lsp_range_from_span(dep.span().clone());
                (dep.clone(), range)
            })
            .collect::<Vec<_>>();

        let mut all_diagnostics = Vec::new();
        let mut fut_diagnostics = Vec::new();
        for (dep, range) in &deps {
            fut_diagnostics.push(diagnose_dependency(
                &self.clients,
                &uri,
                &index_url,
                dep,
                range,
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

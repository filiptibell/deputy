use tracing::trace;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::server::*;
use crate::util::join_all;

use super::*;

mod compat;
mod completion;
mod constants;
mod diagnostics;
mod manifest;

use completion::*;
use diagnostics::*;
use manifest::*;

#[derive(Debug, Clone)]
pub struct Toolchain {
    _client: Client,
    clients: Clients,
    documents: Documents,
}

impl Toolchain {
    pub(super) fn new(client: Client, clients: Clients, documents: Documents) -> Self {
        Self {
            _client: client,
            clients,
            documents,
        }
    }

    fn get_document(&self, uri: &Url) -> Option<(Document, Manifest)> {
        let document = self.documents.get(uri).map(|r| r.clone())?;
        let manifest = Manifest::parse(document.as_str()).ok()?;
        Some((document, manifest))
    }
}

#[tower_lsp::async_trait]
impl Tool for Toolchain {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let (document, manifest) = match self.get_document(&uri) {
            None => return Ok(None),
            Some(d) => d,
        };

        let offset = document.lsp_position_to_offset(pos);
        let found = manifest.tools.iter().find_map(|(_, tool)| {
            let span = tool.span();
            if offset >= span.start && offset <= span.end {
                Some((document.lsp_range_from_span(span.clone()), tool.spec()))
            } else {
                None
            }
        });

        let (found_range, found_spec) = match found {
            Some((range, Ok(spec))) => (range, spec),
            _ => return Ok(None),
        };

        trace!("Hovering: {found_spec:?}");

        let mut lines = Vec::new();
        lines.push(format!("## {}", found_spec.name));
        lines.push(format!(
            "By **{}** - **{}**",
            found_spec.author, found_spec.tag
        ));

        if let Ok(metrics) = self
            .clients
            .github
            .get_repository_metrics(&found_spec.author, &found_spec.name)
            .await
        {
            if let Some(description) = metrics.description {
                lines.push(String::new());
                lines.push(description.to_string());
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

        let (document, manifest) = match self.get_document(&uri) {
            None => return Ok(CompletionResponse::Array(Vec::new())),
            Some(d) => d,
        };

        let offset = document.lsp_position_to_offset(pos);
        let found = manifest.tools.iter().find(|(_, tool)| {
            let span = tool.span();
            offset >= span.start && offset <= span.end
        });
        let found = match found {
            None => return Ok(CompletionResponse::Array(Vec::new())),
            Some(tool) => tool,
        };

        let range_before = document.lsp_range_to_span(Range {
            start: document.lsp_position_from_offset(found.1.span().start + 1),
            end: pos,
        });

        let slice_before = &document.as_str()[range_before.clone()];
        get_tool_completions(&self.clients, &document, range_before, slice_before).await
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        let uri = params.text_document.uri;

        let (document, manifest) = match self.get_document(&uri) {
            None => return Ok(Vec::new()),
            Some(d) => d,
        };

        let tools = manifest
            .tools
            .values()
            .map(|tool| {
                let range = document.lsp_range_from_span(tool.span().clone());
                (tool.clone(), range)
            })
            .collect::<Vec<_>>();

        let mut all_diagnostics = Vec::new();
        let mut fut_diagnostics = Vec::new();
        for (tool, range) in &tools {
            if let Some(diag) = diagnose_tool_spec(tool, range) {
                all_diagnostics.push(diag);
            } else {
                let fut = diagnose_tool_version(&self.clients, &uri, tool, range);
                fut_diagnostics.push(fut);
            }
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

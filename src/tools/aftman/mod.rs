use std::sync::Arc;

use futures::future::join_all;
use tracing::trace;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::github::GithubWrapper;

use crate::server::*;

use super::shared::actions::*;
use super::shared::completion::*;
use super::shared::diagnostics::*;
use super::shared::manifest::*;
use super::*;

#[derive(Debug, Clone)]
pub struct Aftman {
    client: Client,
    github: GithubWrapper,
    documents: Documents,
}

impl Aftman {
    pub(super) fn new(client: Client, github: GithubWrapper, documents: Documents) -> Self {
        Self {
            client,
            github,
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
impl Tool for Aftman {
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
        let found = manifest.tools_map.tools.iter().find_map(|tool| {
            if offset >= tool.val_span.start && offset <= tool.val_span.end {
                Some((
                    document.lsp_range_from_range(tool.val_span.clone()),
                    tool.spec(),
                ))
            } else {
                None
            }
        });

        let (found_range, found_spec) = match found {
            Some((range, Ok(spec))) => (range, spec),
            _ => return Ok(None),
        };

        trace!("Hovering: {found_spec}");

        let mut lines = Vec::new();
        lines.push(format!("## {}", found_spec.name));
        lines.push(format!(
            "By **{}** - **{}**",
            found_spec.author, found_spec.version
        ));

        if let Ok(metrics) = self
            .github
            .get_repository_metrics(found_spec.author, found_spec.name)
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

    async fn completion(&self, params: CompletionParams) -> Result<Vec<CompletionItem>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let document = match self.get_document(&uri).await {
            None => return Ok(Vec::new()),
            Some(d) => d,
        };
        let manifest = match Manifest::parse(document.as_str()) {
            Err(_) => return Ok(Vec::new()),
            Ok(m) => m,
        };

        let offset = document.lsp_position_to_offset(pos);
        let found = manifest
            .tools_map
            .tools
            .iter()
            .find(|tool| offset >= tool.val_span.start && offset <= tool.val_span.end);
        let found = match found {
            None => return Ok(Vec::new()),
            Some(tool) => tool,
        };

        let range_before = document.lsp_range_to_range(Range {
            start: document.lsp_position_from_offset(found.val_span.start + 1),
            end: pos,
        });

        let slice_before = &document.as_str()[range_before.clone()];
        get_tool_completions(&self.github, &document, range_before, slice_before).await
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        let uri = params.text_document.uri;

        let document = match self.get_document(&uri).await {
            None => return Ok(Vec::new()),
            Some(d) => d,
        };
        let manifest = match Manifest::parse(document.as_str()) {
            Err(_) => return Ok(Vec::new()),
            Ok(m) => m,
        };

        let tools = manifest
            .tools_map
            .tools
            .iter()
            .map(|tool| {
                let range = document.lsp_range_from_range(tool.val_span.clone());
                (tool.clone(), range)
            })
            .collect::<Vec<_>>();

        let mut all_diagnostics = Vec::new();
        let mut fut_diagnostics = Vec::new();
        for (tool, range) in &tools {
            if let Some(diag) = diagnose_tool_spec(tool, range) {
                all_diagnostics.push(diag);
            } else {
                let fut = diagnose_tool_version(&self.github, &uri, tool, range);
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
            if let Some(Ok(action)) = diag.data.as_ref().map(CodeActionMetadata::try_from) {
                actions.push(action.into_code_action(diag.clone()))
            }
        }
        Ok(actions)
    }
}

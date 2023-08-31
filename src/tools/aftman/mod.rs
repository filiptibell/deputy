use std::sync::Arc;

use futures::future::join_all;
use tracing::trace;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::github::GithubWrapper;

use crate::server::*;
use crate::util::*;

use super::shared::actions::*;
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
}

#[tower_lsp::async_trait]
impl Tool for Aftman {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let documents = Arc::clone(&self.documents);
        let documents = documents.lock().await;

        let document = match documents.get(&uri) {
            None => return Ok(None),
            Some(d) => d,
        };
        let manifest = match Manifest::parse(&document.text) {
            Err(_) => return Ok(None),
            Ok(m) => m,
        };

        let offset = position_to_offset(&manifest.source, pos);
        let found = manifest.tools_map.tools.iter().find_map(|tool| {
            if offset >= tool.val_span.start && offset <= tool.val_span.end {
                Some((
                    offset_range_to_range(&manifest.source, tool.val_span.clone()),
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

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        let uri = params.text_document.uri;

        let documents = Arc::clone(&self.documents);
        let documents = documents.lock().await;

        let document = match documents.get(&uri) {
            None => return Ok(Vec::new()),
            Some(d) => d,
        };
        let manifest = match Manifest::parse(&document.text) {
            Err(_) => return Ok(Vec::new()),
            Ok(m) => m,
        };

        let tools = manifest
            .tools_map
            .tools
            .iter()
            .map(|tool| {
                let range = offset_range_to_range(&manifest.source, tool.val_span.clone());
                (tool.clone(), range)
            })
            .collect::<Vec<_>>();

        let mut diagnostics = Vec::new();
        for (tool, range) in &tools {
            if let Some(diag) = diagnose_tool_spec(tool, range) {
                diagnostics.push(diag);
            }
        }

        // Return parsing errors immediately - leads to better responsiveness
        // for parsing errors since we don't wait on other (potentially valid)
        // tools to fetch their info to show new parsing errors when typing
        if !diagnostics.is_empty() {
            return Ok(diagnostics);
        }

        let diags = tools
            .iter()
            .map(|(tool, range)| diagnose_tool_version(&self.github, &uri, tool, range))
            .collect::<Vec<_>>();

        let mut diags = join_all(diags).await;
        for diag in diags.drain(..).flatten() {
            diagnostics.push(diag);
        }
        Ok(diagnostics)
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

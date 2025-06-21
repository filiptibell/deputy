use async_language_server::{
    lsp_types::{
        ClientCapabilities, CodeActionKind, CodeActionOptions, CodeActionParams,
        CodeActionProviderCapability, CodeActionResponse, CompletionOptions, CompletionParams,
        CompletionResponse, DiagnosticOptions, DiagnosticServerCapabilities,
        DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
        FullDocumentDiagnosticReport, Hover, HoverParams, HoverProviderCapability,
        RelatedFullDocumentDiagnosticReport, ServerCapabilities, ServerInfo,
    },
    server::{DocumentMatcher, Server, ServerResult, ServerState},
};

use crate::{clients::Clients, tools::Tools};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToolingLanguageServer {
    clients: Clients,
    tools: Tools,
}

impl ToolingLanguageServer {
    pub fn new() -> Self {
        let clients = Clients::new();
        let tools = Tools::new(&clients);
        Self { clients, tools }
    }
}

impl Default for ToolingLanguageServer {
    fn default() -> Self {
        Self::new()
    }
}

impl Server for ToolingLanguageServer {
    fn server_info() -> Option<ServerInfo> {
        Some(ServerInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        })
    }

    fn server_capabilities(_: ClientCapabilities) -> Option<ServerCapabilities> {
        Some(ServerCapabilities {
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(completion_trigger_characters()),
                ..Default::default()
            }),
            code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                resolve_provider: Some(false),
                ..Default::default()
            })),
            diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                inter_file_dependencies: false,
                workspace_diagnostics: false,
                ..Default::default()
            })),
            ..Default::default()
        })
    }

    fn server_document_matchers() -> Vec<DocumentMatcher> {
        let matchers = [
            (
                "Cargo",
                ["**/Cargo.toml", "Cargo.toml"],
                tree_sitter_toml_ng::LANGUAGE,
            ),
            (
                "NPM",
                ["**/package.json", "package.json"],
                tree_sitter_json::LANGUAGE,
            ),
            (
                "Rokit",
                ["**/rokit.toml", "rokit.toml"],
                tree_sitter_toml_ng::LANGUAGE,
            ),
            (
                "Wally",
                ["**/wally.toml", "wally.toml"],
                tree_sitter_toml_ng::LANGUAGE,
            ),
        ];

        matchers
            .into_iter()
            .map(|(name, globs, lang)| {
                DocumentMatcher::new(name)
                    .with_url_globs(globs)
                    .with_lang_grammar(lang.into())
            })
            .collect()
    }

    async fn hover(&self, state: ServerState, params: HoverParams) -> ServerResult<Option<Hover>> {
        let url = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let Some(doc) = state.document(&url) else {
            return Ok(None);
        };
        let Some(node) = doc.node_at_position_named(pos) else {
            tracing::debug!(
                "Missing node for hover at {}:{} (document matcher: {})",
                pos.line,
                pos.character,
                doc.matched_name().unwrap_or("None")
            );
            return Ok(None);
        };

        tracing::debug!("Getting hover for node at {}:{}", pos.line, pos.character);

        self.tools.hover(&doc, pos, node).await
    }

    async fn completion(
        &self,
        state: ServerState,
        params: CompletionParams,
    ) -> ServerResult<Option<CompletionResponse>> {
        let url = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;

        let Some(doc) = state.document(&url) else {
            return Ok(None);
        };
        let Some(node) = doc.node_at_position_named(pos) else {
            tracing::debug!(
                "Missing node for completion at {}:{} (document matcher: {})",
                pos.line,
                pos.character,
                doc.matched_name().unwrap_or("None")
            );
            return Ok(None);
        };

        tracing::debug!(
            "Getting completions for node at {}:{}",
            pos.line,
            pos.character
        );

        self.tools.completion(&doc, pos, node).await
    }

    async fn document_diagnostics(
        &self,
        state: ServerState,
        params: DocumentDiagnosticParams,
    ) -> ServerResult<DocumentDiagnosticReportResult> {
        let items = match state.document(&params.text_document.uri) {
            Some(doc) => self.tools.diagnostics(&doc, params).await?,
            None => Vec::new(),
        };

        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items,
                },
            }),
        ))
    }

    async fn code_action(
        &self,
        state: ServerState,
        params: CodeActionParams,
    ) -> ServerResult<Option<CodeActionResponse>> {
        if let Some(doc) = state.document(&params.text_document.uri) {
            self.tools.code_action(&doc, params).await.map(Some)
        } else {
            Ok(None)
        }
    }
}

pub fn completion_trigger_characters() -> Vec<String> {
    let mut chars = vec![
        String::from("\""),
        String::from("'"),
        String::from("/"),
        String::from("@"),
        String::from("."),
        String::from("-"),
        String::from("_"),
    ];

    chars.sort();
    chars
}

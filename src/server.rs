use async_language_server::{
    lsp_types::{
        ClientCapabilities, CompletionOptions, CompletionParams, CompletionResponse, Hover,
        HoverParams, HoverProviderCapability, ServerCapabilities, ServerInfo,
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
        let tools = Tools::new(clients.clone());
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
            ..Default::default()
        })
    }

    fn server_document_matchers() -> Vec<DocumentMatcher> {
        let matchers = [
            ("Cargo", "Cargo.toml", tree_sitter_toml_ng::LANGUAGE),
            ("NPM", "package.json", tree_sitter_json::LANGUAGE),
            ("Rokit", "rokit.toml", tree_sitter_toml_ng::LANGUAGE),
            ("Wally", "wally.toml", tree_sitter_toml_ng::LANGUAGE),
        ];

        matchers
            .into_iter()
            .map(|(name, filename, lang)| {
                DocumentMatcher::new(name)
                    .with_url_globs([filename.to_string()])
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
            tracing::debug!("Missing node for hover at {}:{}", pos.line, pos.character);
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
                "Missing node for completion at {}:{}",
                pos.line,
                pos.character
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

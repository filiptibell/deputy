use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tracing::warn;

use crate::clients::*;
use crate::server::*;

// Tools modules

mod name;
mod shared;
mod tool;

use name::*;
use shared::*;
pub use tool::*;

// Individual tools

mod cargo;
mod npm;
mod rokit;
mod wally;

use cargo::*;
use npm::*;
use rokit::*;
use wally::*;

// Tools manager

#[derive(Debug, Clone)]
pub struct Tools {
    cargo: Cargo,
    npm: Npm,
    rokit: Rokit,
    wally: Wally,
}

impl Tools {
    pub fn new(client: Client, clients: Clients, documents: Documents) -> Self {
        Self {
            cargo: Cargo::new(client.clone(), clients.clone(), documents.clone()),
            npm: Npm::new(client.clone(), clients.clone(), documents.clone()),
            rokit: Rokit::new(client.clone(), clients.clone(), documents.clone()),
            wally: Wally::new(client.clone(), clients.clone(), documents.clone()),
        }
    }

    pub fn file_globs() -> Vec<&'static str> {
        ToolName::all().into_iter().map(|t| t.file_glob()).collect()
    }

    pub fn relevant_file_uris(uri: &Url) -> Vec<Url> {
        ToolName::all()
            .into_iter()
            .flat_map(|t| t.relevant_file_uris(uri))
            .collect()
    }

    fn tool_for_uri(&self, uri: &Url) -> Option<&dyn Tool> {
        match ToolName::from_uri(uri) {
            Ok(ToolName::Aftman) => Some(&self.rokit),
            Ok(ToolName::Cargo) => Some(&self.cargo),
            Ok(ToolName::Npm) => Some(&self.npm),
            Ok(ToolName::Rokit) => Some(&self.rokit),
            Ok(ToolName::Wally) => Some(&self.wally),
            Err(e) => {
                warn!("Failed to parse tool name from uri '{e}'");
                None
            }
        }
    }
}

#[tower_lsp::async_trait]
impl Tool for Tools {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        match self.tool_for_uri(&params.text_document_position_params.text_document.uri) {
            Some(tool) => tool.hover(params).await,
            None => Ok(None),
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
        match self.tool_for_uri(&params.text_document_position.text_document.uri) {
            Some(tool) => tool.completion(params).await,
            None => Ok(CompletionResponse::Array(Vec::new())),
        }
    }

    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        match item.data.as_ref().map(ResolveContextPartial::try_from) {
            Some(Ok(context)) => match self.tool_for_uri(&context.uri) {
                Some(tool) => tool.completion_resolve(item).await,
                None => Ok(item),
            },
            _ => Ok(item),
        }
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        match self.tool_for_uri(&params.text_document.uri) {
            Some(tool) => tool.diagnostics(params).await,
            None => Ok(Vec::new()),
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>> {
        match self.tool_for_uri(&params.text_document.uri) {
            Some(tool) => tool.code_action(params).await,
            None => Ok(Vec::new()),
        }
    }

    async fn code_action_resolve(&self, action: CodeAction) -> Result<CodeAction> {
        match action.data.as_ref().map(ResolveContextPartial::try_from) {
            Some(Ok(context)) => match self.tool_for_uri(&context.uri) {
                Some(tool) => tool.code_action_resolve(action).await,
                None => Ok(action),
            },
            _ => Ok(action),
        }
    }
}

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tracing::warn;

use crate::crates::CratesWrapper;
use crate::github::GithubWrapper;
use crate::server::*;

use super::cargo::*;
use super::toolchain::*;
use super::wally::*;
use super::*;

#[derive(Debug, Clone)]
pub struct Tools {
    cargo: Cargo,
    toolchain: Toolchain,
    wally: Wally,
}

impl Tools {
    pub fn new(
        client: Client,
        documents: Documents,
        github: GithubWrapper,
        crates: CratesWrapper,
    ) -> Self {
        Self {
            cargo: Cargo::new(client.clone(), documents.clone(), crates.clone()),
            toolchain: Toolchain::new(client.clone(), documents.clone(), github.clone()),
            wally: Wally::new(client.clone(), documents.clone(), github.clone()),
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
}

#[tower_lsp::async_trait]
impl Tool for Tools {
    fn affects(&self, name: impl AsRef<str>) -> bool {
        name.as_ref().parse::<ToolName>().is_ok()
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        match ToolName::from_uri(&params.text_document_position_params.text_document.uri) {
            Ok(ToolName::Aftman) => self.toolchain.hover(params).await,
            Ok(ToolName::Cargo) => self.cargo.hover(params).await,
            Ok(ToolName::Foreman) => self.toolchain.hover(params).await,
            Ok(ToolName::Wally) => self.wally.hover(params).await,
            Err(e) => {
                warn!("Failed to parse file name from uri '{e}'");
                Ok(None)
            }
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
        match ToolName::from_uri(&params.text_document_position.text_document.uri) {
            Ok(ToolName::Aftman) => self.toolchain.completion(params).await,
            Ok(ToolName::Cargo) => self.cargo.completion(params).await,
            Ok(ToolName::Foreman) => self.toolchain.completion(params).await,
            Ok(ToolName::Wally) => self.wally.completion(params).await,
            Err(e) => {
                warn!("Failed to parse file name from uri '{e}'");
                Ok(CompletionResponse::Array(Vec::new()))
            }
        }
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        match ToolName::from_uri(&params.text_document.uri) {
            Ok(ToolName::Aftman) => self.toolchain.diagnostics(params).await,
            Ok(ToolName::Cargo) => self.cargo.diagnostics(params).await,
            Ok(ToolName::Foreman) => self.toolchain.diagnostics(params).await,
            Ok(ToolName::Wally) => self.wally.diagnostics(params).await,
            Err(e) => {
                warn!("Failed to parse file name from uri '{e}'");
                Ok(Vec::new())
            }
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>> {
        match ToolName::from_uri(&params.text_document.uri) {
            Ok(ToolName::Aftman) => self.toolchain.code_action(params).await,
            Ok(ToolName::Cargo) => self.cargo.code_action(params).await,
            Ok(ToolName::Foreman) => self.toolchain.code_action(params).await,
            Ok(ToolName::Wally) => self.wally.code_action(params).await,
            Err(e) => {
                warn!("Failed to parse file name from uri '{e}'");
                Ok(Vec::new())
            }
        }
    }
}

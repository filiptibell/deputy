use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::github::GithubWrapper;
use crate::server::*;

use super::*;

#[derive(Debug, Clone)]
pub struct Tools {
    aftman: aftman::Aftman,
    foreman: foreman::Foreman,
    wally: wally::Wally,
}

impl Tools {
    pub fn new(client: Client, github: GithubWrapper, documents: Documents) -> Self {
        Self {
            aftman: aftman::Aftman::new(client.clone(), github.clone(), documents.clone()),
            foreman: foreman::Foreman::new(client.clone(), github.clone(), documents.clone()),
            wally: wally::Wally::new(client.clone(), github.clone(), documents.clone()),
        }
    }

    pub fn file_globs() -> Vec<&'static str> {
        ToolName::all().into_iter().map(|t| t.file_glob()).collect()
    }
}

#[tower_lsp::async_trait]
impl Tool for Tools {
    fn affects(&self, name: impl AsRef<str>) -> bool {
        name.as_ref().parse::<ToolName>().is_ok()
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        match ToolName::from_uri(&params.text_document_position_params.text_document.uri) {
            Ok(ToolName::Aftman) => self.aftman.hover(params).await,
            Ok(ToolName::Foreman) => self.foreman.hover(params).await,
            Ok(ToolName::Wally) => self.wally.hover(params).await,
            Err(_) => Ok(None),
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Vec<CompletionItem>> {
        match ToolName::from_uri(&params.text_document_position.text_document.uri) {
            Ok(ToolName::Aftman) => self.aftman.completion(params).await,
            Ok(ToolName::Foreman) => self.foreman.completion(params).await,
            Ok(ToolName::Wally) => self.wally.completion(params).await,
            Err(_) => Ok(Vec::new()),
        }
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        match ToolName::from_uri(&params.text_document.uri) {
            Ok(ToolName::Aftman) => self.aftman.diagnostics(params).await,
            Ok(ToolName::Foreman) => self.foreman.diagnostics(params).await,
            Ok(ToolName::Wally) => self.wally.diagnostics(params).await,
            Err(_) => Ok(Vec::new()),
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>> {
        match ToolName::from_uri(&params.text_document.uri) {
            Ok(ToolName::Aftman) => self.aftman.code_action(params).await,
            Ok(ToolName::Foreman) => self.foreman.code_action(params).await,
            Ok(ToolName::Wally) => self.wally.code_action(params).await,
            Err(_) => Ok(Vec::new()),
        }
    }
}

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use super::*;

#[derive(Debug, Clone)]
pub struct Tools {
    aftman: aftman::Aftman,
    foreman: foreman::Foreman,
    wally: wally::Wally,
}

#[tower_lsp::async_trait]
impl Tool for Tools {
    fn new() -> Self {
        Self {
            aftman: aftman::Aftman::new(),
            foreman: foreman::Foreman::new(),
            wally: wally::Wally::new(),
        }
    }

    fn affects(&self, name: impl AsRef<str>) -> bool {
        name.as_ref().parse::<ToolName>().is_ok()
    }

    async fn diagnose(
        &self,
        name: String,
        params: DocumentDiagnosticParams,
    ) -> Result<Vec<Diagnostic>> {
        match name.parse::<ToolName>() {
            Ok(ToolName::Aftman) => self.aftman.diagnose(name, params).await,
            Ok(ToolName::Foreman) => self.foreman.diagnose(name, params).await,
            Ok(ToolName::Wally) => self.wally.diagnose(name, params).await,
            Err(_) => Ok(Vec::new()),
        }
    }

    async fn hover(&self, name: String, params: HoverParams) -> Result<Option<Hover>> {
        match name.parse::<ToolName>() {
            Ok(ToolName::Aftman) => self.aftman.hover(name, params).await,
            Ok(ToolName::Foreman) => self.foreman.hover(name, params).await,
            Ok(ToolName::Wally) => self.wally.hover(name, params).await,
            Err(_) => Ok(None),
        }
    }

    async fn completion(&self, name: String, params: CompletionParams) -> Result<Option<Hover>> {
        match name.parse::<ToolName>() {
            Ok(ToolName::Aftman) => self.aftman.completion(name, params).await,
            Ok(ToolName::Foreman) => self.foreman.completion(name, params).await,
            Ok(ToolName::Wally) => self.wally.completion(name, params).await,
            Err(_) => Ok(None),
        }
    }

    async fn code_action(
        &self,
        name: String,
        params: CodeActionParams,
    ) -> Result<Vec<CodeActionOrCommand>> {
        match name.parse::<ToolName>() {
            Ok(ToolName::Aftman) => self.aftman.code_action(name, params).await,
            Ok(ToolName::Foreman) => self.foreman.code_action(name, params).await,
            Ok(ToolName::Wally) => self.wally.code_action(name, params).await,
            Err(_) => Ok(Vec::new()),
        }
    }
}

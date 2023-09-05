use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

#[tower_lsp::async_trait]
pub trait Tool: Send + Sync {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let _params = params;
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
        let _params = params;
        Ok(CompletionResponse::Array(vec![]))
    }

    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        Ok(item)
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        let _params = params;
        Ok(vec![])
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>> {
        let _params = params;
        Ok(vec![])
    }

    async fn code_action_resolve(&self, action: CodeAction) -> Result<CodeAction> {
        Ok(action)
    }
}

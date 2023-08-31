use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

#[tower_lsp::async_trait]
pub trait Tool {
    fn affects(&self, name: impl AsRef<str>) -> bool {
        let _name = name.as_ref();
        false
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let _params = params;
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Vec<CompletionItem>> {
        let _params = params;
        Ok(vec![])
    }

    async fn diagnostics(&self, params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
        let _params = params;
        Ok(vec![])
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Vec<CodeActionOrCommand>> {
        let _params = params;
        Ok(vec![])
    }
}

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

#[tower_lsp::async_trait]
pub trait Tool {
    fn new() -> Self;

    fn affects(&self, name: impl AsRef<str>) -> bool {
        let _name = name.as_ref();
        false
    }

    async fn diagnose(
        &self,
        name: String,
        params: DocumentDiagnosticParams,
    ) -> Result<Vec<Diagnostic>> {
        let _name = name;
        let _params = params;
        Ok(vec![])
    }

    async fn hover(&self, name: String, params: HoverParams) -> Result<Option<Hover>> {
        let _name = name;
        let _params = params;
        Ok(None)
    }

    async fn completion(&self, name: String, params: CompletionParams) -> Result<Option<Hover>> {
        let _name = name;
        let _params = params;
        Ok(None)
    }

    async fn code_action(
        &self,
        name: String,
        params: CodeActionParams,
    ) -> Result<Vec<CodeActionOrCommand>> {
        let _name = name;
        let _params = params;
        Ok(vec![])
    }
}

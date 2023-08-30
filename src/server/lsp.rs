use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;

use super::backend::*;

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.update_all_workspaces().await;
        self.respond_to_initalize(params).await
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.update_document(
            params.text_document.uri,
            params.text_document.version,
            params.text_document.text,
        );
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.update_document(
            params.text_document.uri,
            params.text_document.version,
            params
                .content_changes
                .pop()
                .expect("Missing content changes in change notification")
                .text,
        );
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let position_params = params.text_document_position_params;

        self.respond_to_hover(
            position_params.text_document.uri.clone(),
            position_params.position,
        )
        .await
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<Vec<CodeActionOrCommand>>> {
        self.respond_to_code_action(params).await
    }
}

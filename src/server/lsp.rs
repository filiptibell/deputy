use std::ops::ControlFlow;

use futures::future::BoxFuture;

use async_lsp::{LanguageServer, ResponseError, Result};

use lsp_types::{
    CodeActionOrCommand, CodeActionParams, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, Hover,
    HoverParams, InitializeParams, InitializeResult,
};

use super::backend::*;

impl LanguageServer for Backend {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<Result<()>>;

    fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> BoxFuture<'static, Result<InitializeResult, Self::Error>> {
        if let Some(folders) = &params.workspace_folders {
            self.workspace_folders = folders
                .iter()
                .map(|folder| (folder.name.clone(), folder.uri.clone()))
                .collect();
        }
        self.update_all_workspaces();
        self.respond_to_initalize(params)
    }

    fn did_open(&mut self, params: DidOpenTextDocumentParams) -> ControlFlow<Result<()>> {
        self.update_document(
            params.text_document.uri,
            params.text_document.version,
            params.text_document.text,
        )
    }

    fn did_close(&mut self, _: DidCloseTextDocumentParams) -> ControlFlow<Result<()>> {
        ControlFlow::Continue(())
    }

    fn did_change(&mut self, mut params: DidChangeTextDocumentParams) -> ControlFlow<Result<()>> {
        self.update_document(
            params.text_document.uri,
            params.text_document.version,
            params
                .content_changes
                .pop()
                .expect("Missing content changes in change notification")
                .text,
        )
    }

    fn did_change_configuration(
        &mut self,
        _: DidChangeConfigurationParams,
    ) -> ControlFlow<Result<()>> {
        ControlFlow::Continue(())
    }

    fn hover(
        &mut self,
        params: HoverParams,
    ) -> BoxFuture<'static, Result<Option<Hover>, Self::Error>> {
        let position_params = params.text_document_position_params;

        self.respond_to_hover(
            position_params.text_document.uri.clone(),
            position_params.position,
        )
    }

    fn code_action(
        &mut self,
        params: CodeActionParams,
    ) -> BoxFuture<'static, Result<Option<Vec<CodeActionOrCommand>>, Self::Error>> {
        self.respond_to_code_action(params)
    }
}

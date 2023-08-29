use std::ops::ControlFlow;

use futures::future::BoxFuture;

use async_lsp::{LanguageServer, ResponseError, Result};

use lsp_types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams, Hover,
    HoverParams, InitializeParams, InitializeResult,
};

use super::state::*;

impl LanguageServer for Server {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<Result<()>>;

    fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> BoxFuture<'static, Result<InitializeResult, Self::Error>> {
        self.respond_to_init(params)
    }

    fn did_open(&mut self, params: DidOpenTextDocumentParams) -> ControlFlow<Result<()>> {
        self.update_document(params.text_document.uri, params.text_document.text)
    }

    fn did_change(&mut self, mut params: DidChangeTextDocumentParams) -> ControlFlow<Result<()>> {
        self.update_document(
            params.text_document.uri,
            params.content_changes.pop().unwrap().text,
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
}

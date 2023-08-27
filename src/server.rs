use std::ops::ControlFlow;
use std::time::Duration;

use futures::future::BoxFuture;
use tracing::info;

use async_lsp::{LanguageServer, ResponseError, Result};

use lsp_types::{
    DidChangeConfigurationParams, Hover, HoverContents, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, MarkedString, ServerCapabilities,
};

use crate::state::*;

impl LanguageServer for ServerState {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<Result<()>>;

    fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> BoxFuture<'static, Result<InitializeResult, Self::Error>> {
        info!("Initializing server with params: {params:?}");
        Box::pin(async move {
            Ok(InitializeResult {
                capabilities: ServerCapabilities {
                    hover_provider: Some(HoverProviderCapability::Simple(true)),
                    ..ServerCapabilities::default()
                },
                server_info: None,
                offset_encoding: None,
            })
        })
    }

    fn hover(&mut self, _: HoverParams) -> BoxFuture<'static, Result<Option<Hover>, Self::Error>> {
        let counter = self.counter;
        Box::pin(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "I am hover text {counter}!"
                ))),
                range: None,
            }))
        })
    }

    fn did_change_configuration(
        &mut self,
        _: DidChangeConfigurationParams,
    ) -> ControlFlow<Result<()>> {
        ControlFlow::Continue(())
    }
}

use std::ops::ControlFlow;

use futures::future::BoxFuture;
use tracing::info;

use async_lsp::{LanguageServer, ResponseError, Result};

use lsp_types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams, Hover,
    HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult,
    MarkedString, ServerCapabilities,
};

use crate::state::*;
use crate::util::*;

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

        let manifest_uri = position_params.text_document.uri;
        let manifest = match self.manifests.get(&manifest_uri) {
            None => return Box::pin(async move { Ok(None) }),
            Some(manifest) => manifest,
        };

        let offset = position_to_offset(&manifest.source, position_params.position);
        let found = manifest.tools_map.tools.iter().find_map(|tool| {
            let hovering_key = offset >= tool.key_span.start && offset <= tool.key_span.end;
            let hovering_val = offset >= tool.val_span.start && offset <= tool.val_span.end;

            if hovering_key || hovering_val {
                let spec = tool.val_text.clone();
                let span = if hovering_key {
                    tool.key_span.clone()
                } else {
                    tool.val_span.clone()
                };
                let range = offset_range_to_range(&manifest.source, span);
                Some((range, spec))
            } else {
                None
            }
        });

        Box::pin(async move {
            match found {
                None => Ok(None),
                Some((range, spec)) => {
                    // TODO: Parse spec, fetch info about tool
                    Ok(Some(Hover {
                        range: Some(range),
                        contents: HoverContents::Scalar(MarkedString::String(spec)),
                    }))
                }
            }
        })
    }
}

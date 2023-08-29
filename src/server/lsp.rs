use std::ops::ControlFlow;

use futures::future::BoxFuture;
use tracing::{debug, info, trace};

use async_lsp::{LanguageServer, ResponseError, Result};

use lsp_types::{
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidOpenTextDocumentParams, Hover,
    HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult,
    MarkedString, ServerCapabilities, ServerInfo,
};

use crate::util::negotiation::*;
use crate::util::position::*;

use super::state::*;

impl LanguageServer for Server {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<Result<()>>;

    fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> BoxFuture<'static, Result<InitializeResult, Self::Error>> {
        trace!("Initializing server with params: {params:#?}");
        // Output info about the client
        if let Some(info) = &params.client_info {
            if let Some(version) = &info.version {
                info!("Client connected - {} v{}", info.name, version);
            } else {
                info!("Client connected - {}", info.name);
            }
        }
        // Negotiate encodings, attempting to use utf8 if possible
        let position_encoding = negotiate_position_encoding(&params);
        let offset_encoding = negotiate_offset_encoding(&params);
        info!(
            "Negotiated position encoding \"{}\"",
            position_encoding.as_str()
        );
        info!("Negotiated offset encoding \"{}\"", offset_encoding);
        // TODO: Read known manifest files in workspace folders in the background

        // Respond with negotiated encoding, server info, capabilities
        Box::pin(async move {
            Ok(InitializeResult {
                offset_encoding: Some(offset_encoding),
                server_info: Some(ServerInfo {
                    name: env!("CARGO_PKG_NAME").to_string(),
                    version: Some(env!("CARGO_PKG_VERSION").to_string()),
                }),
                capabilities: ServerCapabilities {
                    hover_provider: Some(HoverProviderCapability::Simple(true)),
                    position_encoding: Some(position_encoding),
                    ..ServerCapabilities::default()
                },
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
            None => {
                debug!(
                    "Got hover request for document {} - no manifest",
                    manifest_uri.path()
                );
                return Box::pin(async move { Ok(None) });
            }
            Some(manifest) => manifest,
        };

        let offset = position_to_offset(&manifest.source, position_params.position);
        let found = manifest.tools_map.tools.iter().find_map(|tool| {
            if offset >= tool.val_span.start && offset <= tool.val_span.end {
                Some((
                    offset_range_to_range(&manifest.source, tool.val_span.clone()),
                    tool.val_text.clone(),
                ))
            } else {
                None
            }
        });

        if found.is_some() {
            debug!(
                "Got hover request for document {} at {}",
                manifest_uri.path(),
                offset
            );
        } else {
            debug!(
                "Got hover request for document {} at {} (tool found)",
                manifest_uri.path(),
                offset
            );
        }

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

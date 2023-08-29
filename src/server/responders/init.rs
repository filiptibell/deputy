use futures::future::BoxFuture;
use itertools::Itertools;
use tracing::{info, trace};

use async_lsp::{ResponseError, Result};
use lsp_types::{
    DiagnosticOptions, DiagnosticServerCapabilities, HoverProviderCapability, InitializeParams,
    InitializeResult, PositionEncodingKind, ServerCapabilities, ServerInfo,
};

use crate::server::Server;

impl Server {
    pub(crate) fn respond_to_init(
        &self,
        params: InitializeParams,
    ) -> BoxFuture<'static, Result<InitializeResult, ResponseError>> {
        trace!("Initializing server with params: {params:#?}");

        // Negotiate with client for settings, encodings, try to prefer using utf-8
        log_client_info(&params);
        let (position_encoding, offset_encoding) = negotiate_position_and_offset_encoding(&params);

        // Read known files in workspace folders in the background
        if let Some(folders) = &params.workspace_folders {
            for folder in folders {
                self.update_workspace_documents(folder.uri.clone());
            }
        }

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
                    diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                        DiagnosticOptions {
                            inter_file_dependencies: false,
                            workspace_diagnostics: false,
                            ..Default::default()
                        },
                    )),
                    ..ServerCapabilities::default()
                },
            })
        })
    }
}

fn log_client_info(params: &InitializeParams) {
    let num_folders = params
        .workspace_folders
        .as_deref()
        .map(|f| f.len())
        .unwrap_or(0);

    if let Some(info) = &params.client_info {
        if let Some(version) = &info.version {
            info!(
                "Client connected - {} v{} - {} workspace folder{}",
                info.name,
                version,
                num_folders,
                if num_folders != 1 { "s" } else { "" }
            );
        } else {
            info!(
                "Client connected - {} - {} workspace folder{}",
                info.name,
                num_folders,
                if num_folders != 1 { "s" } else { "" }
            );
        }
    }
}

fn negotiate_position_and_offset_encoding(
    params: &InitializeParams,
) -> (PositionEncodingKind, String) {
    let encoding_capabilities = match &params.capabilities.general {
        Some(general) => general.position_encodings.as_ref(),
        None => None,
    };

    let position_encodings_str;
    let position_encoding = match encoding_capabilities {
        Some(encodings) => {
            position_encodings_str = encodings
                .iter()
                .map(|e| format!("\"{}\"", e.as_str()))
                .join(", ");
            if encodings.contains(&PositionEncodingKind::UTF8) {
                PositionEncodingKind::UTF8
            } else {
                PositionEncodingKind::UTF16
            }
        }
        None => {
            position_encodings_str = String::from("N/A");
            PositionEncodingKind::UTF16
        }
    };

    let offset_encodings_str;
    let offset_encoding = match &params.capabilities.offset_encoding {
        Some(encodings) => {
            offset_encodings_str = encodings
                .iter()
                .map(|e| format!("\"{}\"", e.as_str()))
                .join(", ");
            if encodings.contains(&String::from("utf-8"))
                || encodings.contains(&String::from("utf8"))
            {
                "utf-8"
            } else {
                "utf-16"
            }
        }
        None => {
            offset_encodings_str = String::from("N/A");
            "utf-16"
        }
    };

    info!(
        "Client encoding support\n\tPosition: {}\n\tOffset: {}",
        position_encodings_str, offset_encodings_str
    );
    info!(
        "Negotiated encodings with client\n\tPosition: \"{}\"\n\tOffset: \"{}\"",
        position_encoding.as_str(),
        offset_encoding,
    );

    (position_encoding, offset_encoding.to_string())
}

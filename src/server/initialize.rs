use tracing::{info, trace};

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::server::*;

fn completion_trigger_characters() -> Vec<String> {
    let mut chars = vec![
        String::from("\""),
        String::from("'"),
        String::from("/"),
        String::from("@"),
        String::from("."),
        String::from("-"),
        String::from("_"),
    ];

    chars.sort();
    chars
}

impl Server {
    pub async fn respond_to_initalize(&self, params: InitializeParams) -> Result<InitializeResult> {
        trace!("Initializing server with params: {params:#?}");

        log_client_info(&params);

        // Create completion provider parameters
        let completion_options = CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(completion_trigger_characters()),
            ..Default::default()
        };

        // Create diagnostic registration parameters combined for all known tools
        let diagnostic_registration_options = DiagnosticRegistrationOptions {
            text_document_registration_options: TextDocumentRegistrationOptions {
                document_selector: Some(
                    Tools::file_globs()
                        .iter()
                        .map(|&glob| DocumentFilter {
                            scheme: Some(String::from("file")),
                            pattern: Some(String::from(glob)),
                            language: None,
                        })
                        .collect(),
                ),
            },
            diagnostic_options: DiagnosticOptions {
                inter_file_dependencies: true,
                workspace_diagnostics: false,
                ..Default::default()
            },
            ..Default::default()
        };

        // Create similar options but for file operation notifications
        let file_operation_options = FileOperationRegistrationOptions {
            filters: Tools::file_globs()
                .iter()
                .map(|&glob| FileOperationFilter {
                    scheme: Some(String::from("file")),
                    pattern: FileOperationPattern {
                        glob: String::from(glob),
                        matches: Some(FileOperationPatternKind::File),
                        options: Some(FileOperationPatternOptions {
                            ignore_case: Some(true),
                        }),
                    },
                })
                .collect(),
        };

        // Respond with negotiated encoding, server info, capabilities
        Ok(InitializeResult {
            offset_encoding: Some(String::from("utf-16")),
            server_info: Some(ServerInfo {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                position_encoding: Some(PositionEncodingKind::UTF16),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        open_close: Some(true),
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(completion_options),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::RegistrationOptions(
                    diagnostic_registration_options,
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                        did_create: Some(file_operation_options.clone()),
                        did_rename: Some(file_operation_options.clone()),
                        did_delete: Some(file_operation_options),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..ServerCapabilities::default()
            },
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

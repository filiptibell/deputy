use std::io;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use futures::future::join_all;
use tokio::fs;
use tokio::time::timeout;
use tower_lsp::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;
use tracing::{info, trace, warn};

use crate::server::conversion::convert_to_utf8;
use crate::server::{DocumentBuilder, Server, Settings};
use crate::tools::{Tool, ToolName, Tools};

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.respond_to_initalize(params).await
    }

    async fn initialized(&self, _: InitializedParams) {
        if self.workspace_diagnostics.is_supported()
            && self
                .client
                .register_capability(vec![Registration {
                    id: "diagnostics".to_string(),
                    method: "textDocument/diagnostic".to_string(),
                    register_options: None,
                }])
                .await
                .is_ok()
        {
            self.workspace_diagnostics.set_enabled(true);
            info!("Workspace diagnostics capability registered");
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        if let Ok(settings) = serde_json::from_value::<Settings>(params.settings) {
            self.settings.update_global_settings(settings);
            if self.workspace_diagnostics.is_enabled() {
                for entry in self.documents.iter() {
                    let uri = entry.key();
                    self.workspace_diagnostics.process(uri).await;
                }
            }
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = &params.text_document.uri;
        if ToolName::from_uri(uri).is_err() {
            return;
        }

        let version = params.text_document.version;
        let text = params.text_document.text.clone();

        let documents = Arc::clone(&self.documents);
        let waiting = self.waiting.clone();

        // Modify any existing file with new version & contents, or insert a new one
        documents
            .entry(uri.clone())
            .and_modify(|document| {
                document.set_version(version);
                document.set_opened(true);
                document.set_text(&text);
            })
            .or_insert_with(|| {
                DocumentBuilder::new()
                    .with_uri(uri.clone())
                    .with_version(version)
                    .with_text(text)
                    .with_opened()
                    .build()
            });
        waiting.trigger(uri.clone());

        // If we have any relevant files, try to read those too right away
        let relevant_uris = Tools::relevant_file_uris(uri)
            .into_iter()
            .filter(|u| !documents.contains_key(u))
            .collect::<Vec<_>>();

        let mut futs = Vec::new();
        for relevant_uri in &relevant_uris {
            let file_path = relevant_uri
                .to_file_path()
                .expect("relevant_file_uris should only return file paths");
            futs.push(async move {
                let bytes = fs::read(&file_path).await?;
                convert_to_utf8(&file_path, &bytes).await
            });
        }

        for (index, result) in join_all(futs).await.into_iter().enumerate() {
            let relevant_uri = relevant_uris.get(index).expect("Missing or unordered uri");
            match result {
                Err(e) => {
                    if e.kind() != io::ErrorKind::NotFound {
                        warn!("Failed to read relevant file at '{uri}' - {e}");
                    }
                }
                Ok(s) => {
                    documents
                        .entry(relevant_uri.clone())
                        .and_modify(|document| {
                            document.set_text(&s);
                        })
                        .or_insert_with(|| {
                            DocumentBuilder::new()
                                .with_uri(relevant_uri.clone())
                                .with_text(s)
                                .with_opened()
                                .build()
                        });
                    waiting.trigger(relevant_uri.clone());
                }
            }
        }

        if self.workspace_diagnostics.is_enabled() {
            self.workspace_diagnostics.process(uri).await;
        }

        trace!("File opened: {uri}");
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = &params.text_document.uri;
        if ToolName::from_uri(uri).is_err() {
            return;
        }

        let documents = Arc::clone(&self.documents);
        let mut document = documents
            .get_mut(uri)
            .expect("Got close event for nonexistent document");
        document.set_opened(false);

        trace!("File closed: {uri}");
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = &params.text_document.uri;
        if ToolName::from_uri(uri).is_err() {
            return;
        }

        let version = params.text_document.version;

        let documents = Arc::clone(&self.documents);
        let mut document = documents
            .get_mut(uri)
            .expect("Got change event for nonexistent document");

        document.set_version(version);
        for change in params.content_changes {
            document.apply_change(change);
        }

        if self.workspace_diagnostics.is_enabled() {
            self.workspace_diagnostics.process(uri).await;
        }

        trace!("File changed: {uri}");
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        let files = params
            .files
            .into_iter()
            .filter(|f| ToolName::from_str(&f.uri).is_ok());

        for create in files {
            let new = Url::parse(create.uri.as_str())
                .expect("Got invalid file path in create notification");
            // NOTE: We intentionally don't read and insert a document here,
            // it is not provided directly in the create files params, and
            // we might as well do it lazily when a file is opened instead
            trace!("File created: {new}");
        }
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        let files = params.files.into_iter().filter(|f| {
            ToolName::from_str(&f.old_uri).is_ok() || ToolName::from_str(&f.new_uri).is_ok()
        });

        let documents = Arc::clone(&self.documents);
        for rename in files {
            let old = Url::parse(rename.old_uri.as_str())
                .expect("Got invalid file path in rename notification");
            let new = Url::parse(rename.new_uri.as_str())
                .expect("Got invalid file path in rename notification");
            if let Some((_, old_doc)) = documents.remove(&old) {
                trace!("File renamed: {old} -> {new}");
                documents.insert(new, old_doc);
            } else {
                warn!("File renamed, but no doc existed: {old} -> {new}")
            }
        }
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        let files = params
            .files
            .into_iter()
            .filter(|f| ToolName::from_str(&f.uri).is_ok());

        let documents = Arc::clone(&self.documents);
        for delete in files {
            let old = Url::parse(delete.uri.as_str())
                .expect("Got invalid file path in delete notification");
            documents.remove(&old);
            trace!("File deleted: {old}");
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        if ToolName::from_uri(uri).is_err() {
            return Ok(None);
        }

        self.wait_if_nonexistent_or_timeout(uri).await?;
        self.tools.hover(params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        if ToolName::from_uri(uri).is_err() {
            return Ok(None);
        }

        self.wait_if_nonexistent_or_timeout(uri).await?;

        match self.tools.completion(params).await {
            Err(e) => Err(e),
            Ok(r) => Ok(Some(r)),
        }
    }

    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        self.tools.completion_resolve(item).await
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let uri = &params.text_document.uri;
        if !self.workspace_diagnostics.can_process(uri) || ToolName::from_uri(uri).is_err() {
            return Ok(DocumentDiagnosticReportResult::Report(
                DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: None,
                        items: Vec::new(),
                    },
                }),
            ));
        }

        self.wait_if_nonexistent_or_timeout(uri).await?;

        match self.tools.diagnostics(params).await {
            Err(e) => Err(e),
            Ok(v) => Ok(DocumentDiagnosticReportResult::Report(
                DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: None,
                        items: v,
                    },
                }),
            )),
        }
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<Vec<CodeActionOrCommand>>> {
        let uri = &params.text_document.uri;
        if ToolName::from_uri(uri).is_err() {
            return Ok(None);
        }

        match self.tools.code_action(params).await {
            Err(e) => Err(e),
            Ok(v) => {
                if v.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(v))
                }
            }
        }
    }

    async fn code_action_resolve(&self, action: CodeAction) -> Result<CodeAction> {
        self.tools.code_action_resolve(action).await
    }

    async fn workspace_diagnostic(
        &self,
        params: WorkspaceDiagnosticParams,
    ) -> Result<WorkspaceDiagnosticReportResult> {
        let uris = self
            .documents
            .iter()
            .filter_map(|entry| {
                let uri = entry.key();
                if self.workspace_diagnostics.can_process(uri) {
                    Some(uri.clone())
                } else {
                    None
                }
            })
            .filter_map(|uri| {
                self.documents
                    .get(&uri)
                    .map(|doc| doc.version())
                    .map(|version| (uri, version, params.identifier.clone()))
            });

        let futs = uris.map(|(uri, version, identifier)| async move {
            let params = DocumentDiagnosticParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                identifier,
                previous_result_id: None,
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            };

            if let Ok(diagnostics) = self.tools.diagnostics(params).await {
                Some(WorkspaceDocumentDiagnosticReport::Full(
                    WorkspaceFullDocumentDiagnosticReport {
                        uri,
                        version: Some(version as i64),
                        full_document_diagnostic_report: FullDocumentDiagnosticReport {
                            result_id: None,
                            items: diagnostics,
                        },
                    },
                ))
            } else {
                None
            }
        });

        let items = join_all(futs)
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(WorkspaceDiagnosticReportResult::Report(
            WorkspaceDiagnosticReport { items },
        ))
    }
}

impl Server {
    async fn wait_if_nonexistent_or_timeout(&self, uri: &Url) -> Result<()> {
        // HACK: Sometimes we receive a notification or request for diagnostics
        // or something similar before the file has been opened, so we need to
        // first wait for it to open and register with the language server

        if self.documents.contains_key(uri) {
            return Ok(());
        }

        let uri = uri.clone();
        let waiting = self.waiting.clone();
        let receiver = waiting.insert(uri.clone());

        match timeout(Duration::from_secs(5), receiver).await {
            Ok(_) => Ok(()),
            Err(_) => {
                waiting.remove(&uri);
                Err(Error::new(ErrorCode::RequestCancelled))
            }
        }
    }
}

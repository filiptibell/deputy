use std::sync::Arc;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;
use tracing::trace;
use tracing::warn;

use super::*;

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.respond_to_initalize(params).await
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;
        let text = params.text_document.text.clone();

        let documents = Arc::clone(&self.documents);
        let mut documents = documents.lock().await;

        let new_document = DocumentBuilder::new()
            .with_uri(uri.clone())
            .with_version(version)
            .with_text(text)
            .build();
        documents.insert(uri, new_document);
    }

    async fn did_close(&self, _params: DidCloseTextDocumentParams) {}

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;

        let documents = Arc::clone(&self.documents);
        let mut documents = documents.lock().await;

        let document = documents
            .get_mut(&uri)
            .expect("Got change event for nonexistent document");

        document.set_version(version);
        for change in params.content_changes {
            document.apply_change(change);
        }
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        for create in params.files {
            let new = Url::parse(create.uri.as_str())
                .expect("Got invalid file path in create notification");
            // NOTE: We intentionally don't read and insert a document here,
            // it is not provided directly in the create files params, and
            // we might as well do it lazily when a file is opened instead
            trace!("File created: {new}");
        }
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        let documents = Arc::clone(&self.documents);
        let mut documents = documents.lock().await;
        for rename in params.files {
            let old = Url::parse(rename.old_uri.as_str())
                .expect("Got invalid file path in rename notification");
            let new = Url::parse(rename.new_uri.as_str())
                .expect("Got invalid file path in rename notification");
            if let Some(old_doc) = documents.remove(&old) {
                trace!("File renamed: {old} -> {new}");
                documents.insert(new, old_doc);
            } else {
                warn!("File renamed, but no doc existed: {old} -> {new}")
            }
        }
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        let documents = Arc::clone(&self.documents);
        let mut documents = documents.lock().await;
        for delete in params.files {
            let old = Url::parse(delete.uri.as_str())
                .expect("Got invalid file path in delete notification");
            documents.remove(&old);
            trace!("File deleted: {old}");
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.tools.hover(params).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        match self.tools.completion(params).await {
            Err(e) => Err(e),
            Ok(r) => Ok(Some(r)),
        }
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
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
}

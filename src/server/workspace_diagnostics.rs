use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};

use tower_lsp::{lsp_types::*, Client};
use url::Url;

use crate::server::{Documents, SettingsMap};
use crate::tools::{Tool, ToolName, Tools};

#[derive(Debug, Clone)]
pub struct WorkspaceDiagnostics {
    supported: Arc<AtomicBool>,
    documents: Documents,
    settings: SettingsMap,
    tools: Tools,
    client: Client,
}

impl WorkspaceDiagnostics {
    pub fn new(client: Client, documents: Documents, settings: SettingsMap, tools: Tools) -> Self {
        Self {
            supported: Arc::new(AtomicBool::new(false)),
            documents,
            settings,
            tools,
            client,
        }
    }

    pub fn set_supported(&self, supported: bool) {
        self.supported.store(supported, Ordering::SeqCst);
    }

    pub fn is_supported(&self) -> bool {
        self.supported.load(Ordering::SeqCst)
    }

    pub fn is_enabled(&self) -> bool {
        self.is_supported() && self.settings.is_workspace_diagnostics_enabled()
    }

    pub fn can_process(&self, uri: &Url) -> bool {
        self.is_enabled() && self.documents.contains_key(uri) && ToolName::from_uri(uri).is_ok()
    }

    pub async fn process(&self, uri: &Url) -> Vec<Diagnostic> {
        if !self.can_process(uri) {
            return Vec::new();
        }

        let version = self.documents.get(uri).map(|doc| doc.version());
        let params = DocumentDiagnosticParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            identifier: None,
            previous_result_id: None,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        if let Ok(diagnostics) = self.tools.diagnostics(params).await {
            self.client
                .publish_diagnostics(uri.clone(), diagnostics.clone(), version)
                .await;
            diagnostics
        } else {
            Vec::new()
        }
    }
}

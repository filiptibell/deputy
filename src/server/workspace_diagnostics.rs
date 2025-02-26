use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};

use tower_lsp::{lsp_types::*, Client};
use tracing::debug;
use url::Url;

use crate::server::{Documents, SettingsMap};
use crate::tools::{Tool, ToolName, Tools};

#[derive(Debug, Clone)]
pub struct WorkspaceDiagnostics {
    supported: Arc<AtomicBool>,
    enabled: Arc<AtomicBool>,
    documents: Documents,
    settings: SettingsMap,
    tools: Tools,
    client: Client,
}

impl WorkspaceDiagnostics {
    pub fn new(client: Client, documents: Documents, settings: SettingsMap, tools: Tools) -> Self {
        Self {
            supported: Arc::new(AtomicBool::new(false)),
            enabled: Arc::new(AtomicBool::new(false)),
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

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn can_process_any(&self) -> bool {
        self.is_supported() && self.is_enabled() && self.settings.is_workspace_diagnostics_enabled()
    }

    pub fn can_process(&self, uri: &Url) -> bool {
        self.is_supported()
            && self.is_enabled()
            && self.settings.is_workspace_diagnostics_enabled_for(uri)
            && self.documents.contains_key(uri)
            && ToolName::from_uri(uri).is_ok()
    }

    pub async fn process(&self, uri: &Url) {
        if !self.can_process(uri) {
            return;
        }

        let params = DocumentDiagnosticParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            identifier: None,
            previous_result_id: None,
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };

        if let Ok(diagnostics) = self.tools.diagnostics(params).await {
            let version = self.documents.get(uri).map(|doc| doc.version());

            self.client
                .publish_diagnostics(uri.clone(), diagnostics, version)
                .await;

            debug!("Published diagnostics for {}", uri);
        }
    }
}

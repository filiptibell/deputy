use std::sync::Arc;

use futures::future::BoxFuture;
use tracing::{debug, trace};

use async_lsp::{ResponseError, Result};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    FullDocumentDiagnosticReport, RelatedFullDocumentDiagnosticReport,
    RelatedUnchangedDocumentDiagnosticReport, UnchangedDocumentDiagnosticReport, Url,
};

use crate::{
    manifest::{Manifest, ManifestTool},
    server::Server,
    util::position::offset_range_to_range,
};

impl Server {
    pub(crate) fn respond_to_document_diagnostic(
        &self,
        uri: Url,
    ) -> BoxFuture<'static, Result<DocumentDiagnosticReportResult, ResponseError>> {
        let manifests = Arc::clone(&self.manifests);
        Box::pin(async move {
            let manifests = manifests.lock().await;

            let manifest = match manifests.get(&uri) {
                None => {
                    debug!(
                        "Got diagnostic request for document {} - no manifest",
                        uri.path()
                    );
                    return Ok(DocumentDiagnosticReportResult::Report(
                        DocumentDiagnosticReport::Unchanged(
                            RelatedUnchangedDocumentDiagnosticReport {
                                related_documents: None,
                                unchanged_document_diagnostic_report:
                                    UnchangedDocumentDiagnosticReport {
                                        result_id: String::from("NO_MANIFEST"),
                                    },
                            },
                        ),
                    ));
                }
                Some(manifest) => manifest,
            };

            debug!("Got diagnostic request for document {}", uri.path());

            let mut items = Vec::new();
            for tool in &manifest.tools_map.tools {
                diagnose_tool_spec(&mut items, manifest, tool);
            }

            Ok(DocumentDiagnosticReportResult::Report(
                DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: None,
                        items,
                    },
                }),
            ))
        })
    }
}

fn diagnose_tool_spec(items: &mut Vec<Diagnostic>, manifest: &Manifest, tool: &ManifestTool) {
    match tool.spec() {
        Err(err) => {
            items.push(Diagnostic {
                source: Some(String::from("Tools")),
                range: offset_range_to_range(&manifest.source, tool.val_span.clone()),
                message: err.to_string(),
                severity: Some(DiagnosticSeverity::ERROR),
                ..Default::default()
            });
        }
        Ok(spec) => {
            trace!("Found tool - {spec}");
        }
    }
}

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
    let idx_slash = tool.val_text.chars().enumerate().find(|(_, c)| c == &'/');
    if idx_slash.is_none() {
        items.push(Diagnostic {
            range: offset_range_to_range(&manifest.source, tool.val_span.clone()),
            message: String::from("Missing tool author"),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
        return;
    }

    let idx_at = tool.val_text.chars().enumerate().find(|(_, c)| c == &'@');
    if idx_at.is_none() {
        items.push(Diagnostic {
            range: offset_range_to_range(&manifest.source, tool.val_span.clone()),
            message: String::from("Missing tool name"),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
        return;
    }

    let idx_slash = idx_slash.unwrap().0;
    let idx_at = idx_at.unwrap().0;

    if idx_at == tool.val_text.len() - 1 {
        items.push(Diagnostic {
            range: offset_range_to_range(&manifest.source, tool.val_span.clone()),
            message: String::from("Missing version"),
            severity: Some(DiagnosticSeverity::ERROR),
            ..Default::default()
        });
        return;
    }

    let tool_author = &tool.val_text[..idx_slash];
    let tool_name = &tool.val_text[idx_slash + 1..idx_at];
    let tool_version = &tool.val_text[idx_at + 1..];

    trace!("Found tool - {tool_author}/{tool_name}@{tool_version}");
}

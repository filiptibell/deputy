use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use async_language_server::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, Range, TextEdit, Url,
    WorkspaceEdit,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CodeActionMetadata {
    LatestVersion {
        edit_range: Range,
        source_uri: Url,
        source_text: String,
        version_current: String,
        version_latest: String,
    },
}

impl CodeActionMetadata {
    pub fn into_code_action(self, diag: Diagnostic) -> CodeActionOrCommand {
        match self {
            Self::LatestVersion {
                edit_range,
                source_uri,
                source_text,
                version_current,
                version_latest,
                ..
            } => {
                let replaced = source_text.replace(&version_current, &version_latest);
                let text_edit = TextEdit {
                    new_text: if replaced == source_text {
                        // failed to replace substring, just insert latest version
                        version_latest
                    } else {
                        // means we replaced substring like ^x0.y0.z0 with ^x1.y1.z1
                        replaced
                    },
                    range: edit_range,
                };
                CodeActionOrCommand::CodeAction(CodeAction {
                    title: String::from("Update to latest version"),
                    kind: Some(CodeActionKind::QUICKFIX),
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(source_uri, vec![text_edit])])),
                        ..Default::default()
                    }),
                    diagnostics: Some(vec![diag]),
                    is_preferred: Some(true),
                    ..Default::default()
                })
            }
        }
    }
}

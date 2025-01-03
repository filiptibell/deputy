use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use tower_lsp::lsp_types::*;

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
                let mut change_map = HashMap::new();
                change_map.insert(
                    source_uri,
                    vec![TextEdit {
                        range: edit_range,
                        new_text: source_text.replace(&version_current, &version_latest),
                    }],
                );
                CodeActionOrCommand::CodeAction(CodeAction {
                    title: String::from("Update to latest version"),
                    kind: Some(CodeActionKind::QUICKFIX),
                    edit: Some(WorkspaceEdit {
                        changes: Some(change_map),
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

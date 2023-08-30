use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use async_lsp::Result;
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, TextEdit, Url, WorkspaceEdit,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CodeActionMetadata {
    LatestVersion {
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
                        range: diag.range,
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

impl TryFrom<JsonValue> for CodeActionMetadata {
    type Error = serde_json::Error;
    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl TryFrom<&JsonValue> for CodeActionMetadata {
    type Error = serde_json::Error;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        serde_json::from_value(value.clone())
    }
}

impl From<CodeActionMetadata> for JsonValue {
    fn from(value: CodeActionMetadata) -> Self {
        serde_json::to_value(value).unwrap()
    }
}

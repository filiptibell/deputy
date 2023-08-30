use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::server::*;

use super::super::actions::*;

impl Backend {
    pub async fn respond_to_code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<Vec<CodeActionOrCommand>>> {
        let mut actions = Vec::new();

        for diag in params.context.diagnostics {
            if let Some(Ok(action)) = diag.data.as_ref().map(CodeActionMetadata::try_from) {
                actions.push(action.into_code_action(diag.clone()))
            }
        }

        Ok(if actions.is_empty() {
            None
        } else {
            Some(actions)
        })
    }
}

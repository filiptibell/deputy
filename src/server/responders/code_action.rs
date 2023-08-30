use futures::future::BoxFuture;

use async_lsp::{ResponseError, Result};
use lsp_types::{CodeActionOrCommand, CodeActionParams};

use crate::server::*;

use super::super::actions::*;

impl Backend {
    pub(crate) fn respond_to_code_action(
        &self,
        params: CodeActionParams,
    ) -> BoxFuture<'static, Result<Option<Vec<CodeActionOrCommand>>, ResponseError>> {
        let mut actions = Vec::new();

        for diag in params.context.diagnostics {
            if let Some(Ok(action)) = diag.data.as_ref().map(CodeActionMetadata::try_from) {
                actions.push(action.into_code_action(diag.clone()))
            }
        }

        Box::pin(async move {
            Ok(if actions.is_empty() {
                None
            } else {
                Some(actions)
            })
        })
    }
}

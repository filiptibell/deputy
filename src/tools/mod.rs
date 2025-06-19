use async_language_server::{
    lsp_types::{
        CodeActionOrCommand, CodeActionParams, CompletionResponse, Diagnostic,
        DocumentDiagnosticParams, Hover, Position,
    },
    server::{Document, ServerResult},
};
use tree_sitter::Node;

use crate::clients::*;

mod cargo;
mod npm;
mod rokit;
mod shared;
mod wally;

use cargo::*;
use npm::*;
use rokit::*;
use shared::*;
use wally::*;

#[derive(Debug, Clone)]
pub struct Tools {
    cargo: Cargo,
    npm: Npm,
    rokit: Rokit,
    wally: Wally,
}

impl Tools {
    pub fn new(clients: Clients) -> Self {
        Self {
            cargo: Cargo::new(clients.clone()),
            npm: Npm::new(clients.clone()),
            rokit: Rokit::new(clients.clone()),
            wally: Wally::new(clients.clone()),
        }
    }

    pub async fn hover(
        &self,
        doc: &Document,
        pos: Position,
        node: Node<'_>,
    ) -> ServerResult<Option<Hover>> {
        let Some(tool) = Tool::from_document(doc) else {
            return Ok(None);
        };

        match tool {
            Tool::Cargo => self.cargo.hover(doc, pos, node).await,
            Tool::Npm => self.npm.hover(doc, pos, node).await,
            Tool::Rokit => self.rokit.hover(doc, pos, node).await,
            Tool::Wally => self.wally.hover(doc, pos, node).await,
        }
    }

    pub async fn completion(
        &self,
        doc: &Document,
        pos: Position,
        node: Node<'_>,
    ) -> ServerResult<Option<CompletionResponse>> {
        let Some(tool) = Tool::from_document(doc) else {
            return Ok(None);
        };

        match tool {
            Tool::Cargo => self.cargo.completion(doc, pos, node).await,
            Tool::Npm => self.npm.completion(doc, pos, node).await,
            Tool::Rokit => self.rokit.completion(doc, pos, node).await,
            Tool::Wally => self.wally.completion(doc, pos, node).await,
        }
    }

    pub async fn diagnostics(
        &self,
        doc: &Document,
        params: DocumentDiagnosticParams,
    ) -> ServerResult<Vec<Diagnostic>> {
        let Some(tool) = Tool::from_document(doc) else {
            return Ok(Vec::new());
        };

        match tool {
            Tool::Cargo => self.cargo.diagnostics(doc, params).await,
            Tool::Npm => self.npm.diagnostics(doc, params).await,
            Tool::Rokit => self.rokit.diagnostics(doc, params).await,
            Tool::Wally => self.wally.diagnostics(doc, params).await,
        }
    }

    pub async fn code_action(
        &self,
        doc: &Document,
        params: CodeActionParams,
    ) -> ServerResult<Vec<CodeActionOrCommand>> {
        if Tool::from_document(doc).is_none() {
            return Ok(Vec::new());
        }

        let mut actions = Vec::new();
        for diag in params.context.diagnostics {
            if let Some(Ok(action)) = diag
                .data
                .as_ref()
                .map(ResolveContext::<CodeActionMetadata>::try_from)
            {
                actions.push(action.into_inner().into_code_action(diag.clone()))
            }
        }

        Ok(actions)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Cargo,
    Npm,
    Rokit,
    Wally,
}

impl Tool {
    fn from_document(doc: &Document) -> Option<Self> {
        match doc.matched_name()?.trim() {
            s if s.eq_ignore_ascii_case("cargo") => Some(Tool::Cargo),
            s if s.eq_ignore_ascii_case("npm") => Some(Tool::Npm),
            s if s.eq_ignore_ascii_case("rokit") => Some(Tool::Rokit),
            s if s.eq_ignore_ascii_case("wally") => Some(Tool::Wally),
            _ => None,
        }
    }
}

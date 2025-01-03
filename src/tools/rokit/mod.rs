use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tracing::debug;

use crate::parser::query_rokit_tools;
use crate::parser::Tool as ParsedTool;
use crate::server::*;
use crate::util::*;

use super::*;

mod hover;

use hover::*;

#[derive(Debug, Clone)]
pub struct Rokit {
    _client: Client,
    clients: Clients,
    documents: Documents,
}

impl Rokit {
    pub(super) fn new(client: Client, clients: Clients, documents: Documents) -> Self {
        Self {
            _client: client,
            clients,
            documents,
        }
    }

    fn get_document(&self, uri: &Url) -> Option<Document> {
        if uri
            .file_name()
            .as_deref()
            .is_some_and(|f| f.eq_ignore_ascii_case("rokit.toml"))
        {
            self.documents.get(uri).map(|r| r.clone())
        } else {
            None
        }
    }
}

#[tower_lsp::async_trait]
impl Tool for Rokit {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(doc) = self.get_document(&uri) else {
            return Ok(None);
        };

        // Find the dependency that is hovered over
        let tools = query_rokit_tools(doc.inner());
        let Some(found) = ParsedTool::find_at_pos(&tools, pos) else {
            return Ok(None);
        };

        // Fetch some extra info and return the hover
        debug!("Hovering: {found:?}");
        get_rokit_hover(&self.clients, &doc, found).await
    }
}

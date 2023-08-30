use std::sync::Arc;

use tracing::trace;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::server::*;
use crate::util::*;

impl Backend {
    pub async fn respond_to_hover(&self, uri: Url, position: Position) -> Result<Option<Hover>> {
        let github = self.github.clone();
        let documents = Arc::clone(&self.documents);

        let documents = documents.lock().await;
        let manifest = match documents.get(&uri) {
            None => return Ok(None),
            Some(doc) => &doc.manifest,
        };

        let offset = position_to_offset(&manifest.source, position);
        let found = manifest.tools_map.tools.iter().find_map(|tool| {
            if offset >= tool.val_span.start && offset <= tool.val_span.end {
                Some((
                    offset_range_to_range(&manifest.source, tool.val_span.clone()),
                    tool.spec(),
                ))
            } else {
                None
            }
        });

        let (found_range, found_spec) = match found {
            Some((range, Ok(spec))) => (range, spec),
            _ => return Ok(None),
        };

        trace!("Hovering: {found_spec}");

        let mut lines = Vec::new();
        lines.push(format!("## {}", found_spec.name));
        lines.push(format!(
            "By **{}** - **{}**",
            found_spec.author, found_spec.version
        ));

        if let Ok(metrics) = github
            .get_repository_metrics(found_spec.author, found_spec.name)
            .await
        {
            if let Some(description) = metrics.description {
                lines.push(String::new());
                lines.push(description.to_string());
            }
        }

        Ok(Some(Hover {
            range: Some(found_range),
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: lines.join("\n"),
            }),
        }))
    }
}

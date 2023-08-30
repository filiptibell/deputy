use std::sync::Arc;

use futures::future::BoxFuture;
use tracing::trace;

use async_lsp::{ResponseError, Result};
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Url};

use crate::server::*;
use crate::util::*;

impl Server {
    pub(crate) fn respond_to_hover(
        &self,
        uri: Url,
        position: Position,
    ) -> BoxFuture<'static, Result<Option<Hover>, ResponseError>> {
        let github = self.github.clone();
        let manifests = Arc::clone(&self.manifests);
        Box::pin(async move {
            let manifests = manifests.lock().await;

            let manifest = match manifests.get(&uri) {
                None => return Ok(None),
                Some(manifest) => manifest,
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
        })
    }
}

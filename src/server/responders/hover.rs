use std::sync::Arc;

use futures::future::BoxFuture;
use tracing::trace;

use async_lsp::{ResponseError, Result};
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Url};

use crate::server::Server;
use crate::util::position::*;

impl Server {
    pub(crate) fn respond_to_hover(
        &self,
        uri: Url,
        position: Position,
    ) -> BoxFuture<'static, Result<Option<Hover>, ResponseError>> {
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

            match found {
                Some((range, Ok(spec))) => {
                    trace!("Hovering: {spec}");

                    // TODO: Fetch info about tool such as desc using the GitHub API

                    let mut lines = Vec::new();
                    lines.push(format!("## {}", spec.name));
                    lines.push(format!("By **{}** - **{}**", spec.author, spec.version));

                    Ok(Some(Hover {
                        range: Some(range),
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: lines.join("\n"),
                        }),
                    }))
                }
                _ => Ok(None),
            }
        })
    }
}

use std::sync::Arc;

use futures::future::BoxFuture;
use tracing::debug;

use async_lsp::{ResponseError, Result};
use lsp_types::{Hover, HoverContents, MarkedString, Position, Url};

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
                None => {
                    debug!(
                        "Got hover request for document {} - no manifest",
                        uri.path()
                    );
                    return Ok(None);
                }
                Some(manifest) => manifest,
            };

            let offset = position_to_offset(&manifest.source, position);
            let found = manifest.tools_map.tools.iter().find_map(|tool| {
                if offset >= tool.val_span.start && offset <= tool.val_span.end {
                    Some((
                        offset_range_to_range(&manifest.source, tool.val_span.clone()),
                        tool.val_text.clone(),
                    ))
                } else {
                    None
                }
            });

            match found {
                None => {
                    debug!(
                        "Got hover request for document {} at {}",
                        uri.path(),
                        offset
                    );
                    Ok(None)
                }
                Some((range, spec)) => {
                    debug!(
                        "Got hover request for document {} at {} (tool found)",
                        uri.path(),
                        offset
                    );
                    // TODO: Parse spec, fetch info about tool
                    Ok(Some(Hover {
                        range: Some(range),
                        contents: HoverContents::Scalar(MarkedString::String(spec)),
                    }))
                }
            }
        })
    }
}

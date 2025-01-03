use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tracing::trace;

use crate::{parser::SimpleDependency, tools::MarkdownBuilder};

use super::{Clients, Document};

pub async fn get_wally_hover(
    clients: &Clients,
    _doc: &Document,
    index_url: &str,
    tool: &SimpleDependency,
) -> Result<Option<Hover>> {
    let Some(spec) = tool.parsed_spec().into_full() else {
        return Ok(None);
    };

    // Modify range to show as hovering over the entire "key = version" pair
    let found_range = Range {
        start: tool.name.range.start,
        end: tool.spec.range.end,
    };

    // Add basic hover information with version and name
    trace!(
        "Hovering: {} version {}",
        spec.name.unquoted(),
        spec.version.unquoted()
    );
    let mut md = MarkdownBuilder::new();

    // Try to fetch additional information from the index - description, links
    trace!("Fetching index metadatas from Wally API");
    if let Ok(mut metadatas) = clients
        .wally
        .get_index_metadatas(index_url, spec.author.unquoted(), spec.name.unquoted())
        .await
    {
        if let Some(metadata) = metadatas.pop() {
            md.h2(metadata.package.name);
            md.version(spec.version.unquoted());

            // Add description, if available
            if let Some(desc) = &metadata.package.description {
                md.br();
                md.p(desc);
            }

            // Add links, if available
            if metadata.package.homepage.is_some() || metadata.package.repository.is_some() {
                md.br();
                md.h3("Links");
                if let Some(homepage) = metadata.package.homepage {
                    md.a("Homepage", homepage);
                }
                if let Some(repository) = metadata.package.repository {
                    md.a("Repository", repository);
                }
            }
        } else {
            md.h2(spec.name.unquoted());
            md.version(spec.version.unquoted());
        }
    } else {
        md.h2(spec.name.unquoted());
        md.version(spec.version.unquoted());
    }

    Ok(Some(Hover {
        range: Some(found_range),
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.build(),
        }),
    }))
}

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tracing::trace;

use crate::{
    parser::SimpleDependency,
    tools::{wally::WALLY_DEFAULT_REGISTRY, MarkdownBuilder},
};

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
        metadatas.reverse(); // Latest last, so we can pop
        if let Some(metadata) = metadatas.pop() {
            md.h2(&metadata.package.name);
            md.version(spec.version.unquoted());

            // Add description, if available
            if let Some(desc) = &metadata.package.description {
                md.br();
                md.p(desc);
            }

            // Add links, if available
            let wally_run = metadata
                .package
                .registry
                .eq_ignore_ascii_case(WALLY_DEFAULT_REGISTRY)
                .then(|| {
                    format!(
                        "https://wally.run/package/{}?version={}",
                        metadata.package.name,
                        spec.version.unquoted()
                    )
                });
            if wally_run.is_some()
                || metadata.package.homepage.is_some()
                || metadata.package.repository.is_some()
            {
                md.br();
                md.h3("Links");
                if let Some(homepage) = metadata.package.homepage {
                    md.a("Homepage", homepage);
                }
                if let Some(repository) = metadata.package.repository {
                    md.a("Repository", repository);
                }
                if let Some(wally) = wally_run {
                    md.a("Wally", wally);
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
        range: Some(tool.range()),
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.build(),
        }),
    }))
}

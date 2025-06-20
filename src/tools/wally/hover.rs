use tracing::trace;

use async_language_server::{
    lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind},
    server::{Document, ServerResult},
    tree_sitter::Node,
    tree_sitter_utils::ts_range_to_lsp_range,
};

use crate::{parser::wally, tools::MarkdownBuilder};

use super::constants::WALLY_DEFAULT_REGISTRY;
use super::Clients;

pub async fn get_wally_hover(
    clients: &Clients,
    doc: &Document,
    index_url: &str,
    node: Node<'_>,
) -> ServerResult<Option<Hover>> {
    let Some(dep) = wally::parse_dependency(node) else {
        return Ok(None);
    };

    let (Some(owner), Some(repository), Some(version)) = dep.spec_ranges(doc).text(doc) else {
        return Ok(None);
    };

    // Add basic hover information with version and name
    trace!("Hovering: {owner} version {version}");
    let mut md = MarkdownBuilder::new();

    // Try to fetch additional information from the index - description, links
    trace!("Fetching index metadatas from Wally API");
    if let Ok(mut metadatas) = clients
        .wally
        .get_index_metadatas(index_url, owner, repository)
        .await
    {
        metadatas.reverse(); // Latest last, so we can pop
        if let Some(metadata) = metadatas.pop() {
            md.h2(&metadata.package.name);
            md.version(version);

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
                        "https://wally.run/package/{}?version={version}",
                        metadata.package.name,
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
            md.h2(repository);
            md.version(version);
        }
    } else {
        md.h2(repository);
        md.version(version);
    }

    Ok(Some(Hover {
        range: Some(ts_range_to_lsp_range(node.range())),
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.build(),
        }),
    }))
}

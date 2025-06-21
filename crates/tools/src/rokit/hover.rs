use tracing::trace;

use async_language_server::{
    lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind},
    server::{Document, ServerResult},
    tree_sitter::Node,
    tree_sitter_utils::ts_range_to_lsp_range,
};

use parser::rokit;

use crate::shared::MarkdownBuilder;

use super::Clients;

pub async fn get_rokit_hover(
    clients: &Clients,
    doc: &Document,
    node: Node<'_>,
) -> ServerResult<Option<Hover>> {
    let Some(dep) = rokit::parse_dependency(node) else {
        return Ok(None);
    };

    let (Some(owner), Some(repository), Some(version)) = dep.spec_ranges(doc).text(doc) else {
        return Ok(None);
    };

    // Add basic hover information with version and name
    trace!("Hovering: {owner} version {version}");
    let mut md = MarkdownBuilder::new();
    md.h2(repository);
    md.version(version);

    // Try to fetch additional information from the index - description, links
    trace!("Fetching repository metrics from GitHub");
    if let Ok(repository) = clients
        .github
        .get_repository_metrics(owner, repository)
        .await
    {
        // Add description, if available
        if let Some(desc) = &repository.description {
            md.br();
            md.p(desc);
        }
    }

    // Add link to the repository and latest release
    md.br();
    md.h3("Links");
    md.a(
        "Repository",
        format!("https://github.com/{owner}/{repository}"),
    );
    md.a(
        "Latest Release",
        format!("https://github.com/{owner}/{repository}/releases/latest"),
    );

    Ok(Some(Hover {
        range: Some(ts_range_to_lsp_range(node.range())),
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.build(),
        }),
    }))
}

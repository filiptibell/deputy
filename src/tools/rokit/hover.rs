use tracing::trace;

use async_language_server::{
    lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind},
    server::{Document, ServerResult},
};

use crate::{parser::SimpleDependency, tools::MarkdownBuilder};

use super::Clients;

pub async fn get_rokit_hover(
    clients: &Clients,
    _doc: &Document,
    tool: &SimpleDependency,
) -> ServerResult<Option<Hover>> {
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
    md.h2(spec.name.unquoted());
    md.version(spec.version.unquoted());

    // Try to fetch additional information from the index - description, links
    trace!("Fetching repository metrics from GitHub");
    if let Ok(repository) = clients
        .github
        .get_repository_metrics(spec.author.unquoted(), spec.name.unquoted())
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
        format!(
            "https://github.com/{}/{}",
            spec.author.unquoted(),
            spec.name.unquoted()
        ),
    );
    md.a(
        "Latest Release",
        format!(
            "https://github.com/{}/{}/releases/latest",
            spec.author.unquoted(),
            spec.name.unquoted()
        ),
    );

    Ok(Some(Hover {
        range: Some(tool.range()),
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.build(),
        }),
    }))
}

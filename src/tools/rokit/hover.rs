use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tracing::trace;

use crate::{parser::SimpleDependency, tools::MarkdownBuilder};

use super::{Clients, Document};

pub async fn get_rokit_hover(
    clients: &Clients,
    _doc: &Document,
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
        range: Some(found_range),
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.build(),
        }),
    }))
}

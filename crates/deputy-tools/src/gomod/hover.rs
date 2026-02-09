use async_language_server::{
    lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind},
    server::{Document, ServerResult},
    tree_sitter::Node,
    tree_sitter_utils::ts_range_to_lsp_range,
};
use tracing::trace;

use deputy_parser::gomod;

use crate::shared::MarkdownBuilder;

use super::Clients;

pub async fn get_gomod_hover(
    clients: &Clients,
    doc: &Document,
    node: Node<'_>,
) -> ServerResult<Option<Hover>> {
    let Some(dep) = gomod::parse_dependency(node) else {
        return Ok(None);
    };

    let (path, version) = dep.text(doc);

    // Add basic hover information with path and version
    trace!(
        "Hovering: {path} version {}",
        version.as_deref().unwrap_or("*")
    );
    let mut md = MarkdownBuilder::new();
    md.h2(&path);
    if let Some(version) = &version {
        md.version(version);
    }

    // Try to fetch description from GitHub (only works for github.com/... modules)
    if let Ok(metrics) = clients.golang.get_module_metadata(&path).await
        && let Some(desc) = &metrics.description
    {
        md.br();
        md.p(desc);
    }

    // Add links
    md.br();
    md.h3("Links");
    md.a("Documentation", format!("https://pkg.go.dev/{path}"));

    // Add GitHub link if it's a GitHub module
    if path.starts_with("github.com/") {
        let parts: Vec<&str> = path.splitn(4, '/').collect();
        if parts.len() >= 3 {
            md.a(
                "Repository",
                format!("https://{}/{}", parts[0], parts[1..3].join("/")),
            );
        }
    }

    Ok(Some(Hover {
        range: Some(ts_range_to_lsp_range(node.range())),
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.build(),
        }),
    }))
}

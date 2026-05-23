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

    // Try to fetch package information from pkg.go.dev
    if let Ok(package) = clients.golang.get_package(&path).await
        && !package.synopsis.is_empty()
    {
        md.br();
        md.p(package.synopsis);
    }

    let module = clients.golang.get_module(&path).await.ok();

    // Add links
    md.br();
    md.h3("Links");
    md.a("Documentation", format!("https://pkg.go.dev/{path}"));

    if let Some(repo_url) = module.and_then(|module| module.repo_url) {
        md.a("Repository", repo_url);
    }

    Ok(Some(Hover {
        range: Some(ts_range_to_lsp_range(node.range())),
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.build(),
        }),
    }))
}

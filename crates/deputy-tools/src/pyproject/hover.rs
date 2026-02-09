use async_language_server::{
    lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind},
    server::{Document, ServerResult},
    tree_sitter::Node,
    tree_sitter_utils::ts_range_to_lsp_range,
};
use tracing::trace;

use deputy_parser::pyproject;

use crate::shared::MarkdownBuilder;

use super::Clients;

pub async fn get_pyproject_hover(
    clients: &Clients,
    doc: &Document,
    node: Node<'_>,
) -> ServerResult<Option<Hover>> {
    let Some(dep) = pyproject::parse_dependency(node) else {
        return Ok(None);
    };

    let (Some(name), version) = dep.text(doc) else {
        return Ok(None);
    };

    // Add basic hover information with version and name
    trace!(
        "Hovering: {name} version {}",
        version.as_deref().unwrap_or("*")
    );
    let mut md = MarkdownBuilder::new();
    md.h2(&name);
    if let Some(version) = &version {
        md.version(version);
    }

    // Try to fetch additional information - description, links
    trace!("Fetching package data from PyPI");
    if let Ok(meta) = clients.pypi.get_registry_metadata(&name).await {
        if let Some(summary) = meta.info.summary.as_deref() {
            md.br();
            md.p(summary);
        }

        // Collect links from project_urls and top-level fields
        let mut repo = None;
        let mut docs = None;
        let mut page = meta.info.home_page.as_deref();

        if let Some(project_urls) = &meta.info.project_urls {
            for (key, url) in project_urls {
                let key_lower = key.to_ascii_lowercase();
                if repo.is_none()
                    && (key_lower.contains("repository")
                        || key_lower.contains("source")
                        || key_lower == "github")
                {
                    repo = Some(url.as_str());
                } else if docs.is_none()
                    && (key_lower.contains("documentation") || key_lower.contains("docs"))
                {
                    docs = Some(url.as_str());
                } else if page.is_none()
                    && (key_lower.contains("homepage") || key_lower.contains("home"))
                {
                    page = Some(url.as_str());
                }
            }
        }

        // Deduplicate: ignore homepage or docs if same as repo
        if page == repo {
            page = None;
        }
        if docs == repo {
            docs = None;
        }

        if repo.is_some() || docs.is_some() || page.is_some() {
            md.br();
            md.h3("Links");
            if let Some(docs) = docs {
                md.a("Documentation", docs);
            }
            if let Some(repo) = repo {
                md.a("Repository", repo);
            }
            if let Some(page) = page {
                md.a("Homepage", page);
            }
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

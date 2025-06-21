use tracing::trace;

use async_language_server::{
    lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind},
    server::{Document, ServerResult},
    tree_sitter::Node,
    tree_sitter_utils::ts_range_to_lsp_range,
};

use clients::npm::models::RegistryMetadataRepositoryVariant;
use parser::npm;
use shared::{VersionReqExt, Versioned};

use crate::shared::MarkdownBuilder;

use super::Clients;

pub async fn get_npm_hover(
    clients: &Clients,
    doc: &Document,
    node: Node<'_>,
) -> ServerResult<Option<Hover>> {
    let Some(dep) = npm::parse_dependency(node) else {
        return Ok(None);
    };

    let (name, spec) = dep.text(doc);
    let Ok(version_req) = spec.parse_version_req() else {
        return Ok(None);
    };

    let version = version_req.minimum_version();

    // Add basic hover information with version and name
    trace!("Hovering: {name} version {version}");
    let mut md = MarkdownBuilder::new();
    md.h2(&name);
    md.version(version);

    // Try to fetch additional information from the index - description, links
    trace!("Fetching package data from npm");
    if let Ok(meta) = clients.npm.get_registry_metadata(&name).await {
        if let Some(desc) = meta.current_version.description.as_ref() {
            md.br();
            md.p(desc);
        }

        // Ignore homepage or docs if it's the same as the repo
        let mut page = meta.current_version.homepage.as_deref();
        let repo = meta.current_version.repository.as_ref();
        if page.is_some_and(|p| {
            repo.is_some_and(|r| r.url().is_some_and(|u| u.eq_ignore_ascii_case(p)))
        }) {
            page = None;
        }

        // Add links to repo and homepage
        md.br();
        md.h3("Links");
        if let Some(repo) = repo.and_then(RegistryMetadataRepositoryVariant::url) {
            md.a("Repository", repo);
        }
        if let Some(page) = page {
            md.a("Homepage", page);
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

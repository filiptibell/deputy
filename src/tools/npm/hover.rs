use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tracing::trace;

use crate::{
    parser::Dependency,
    tools::MarkdownBuilder,
    util::{VersionReqExt, Versioned},
};

use super::{Clients, Document};

pub async fn get_npm_hover(
    clients: &Clients,
    _doc: &Document,
    dep: &Dependency,
) -> Result<Option<Hover>> {
    let Ok(version_req) = dep.parse_version_req() else {
        return Ok(None);
    };

    let dependency_name = dep.name().unquoted();
    let dependency_version = version_req.minimum_version();

    // Add basic hover information with version and name
    trace!("Hovering: {dependency_name} version {dependency_version}");
    let mut md = MarkdownBuilder::new();
    md.h2(dependency_name);
    md.version(dependency_version);

    // Try to fetch additional information from the index - description, links
    trace!("Fetching package data from npm");
    if let Ok(meta) = clients.npm.get_registry_metadata(dependency_name).await {
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
        if let Some(repo) = repo.and_then(|r| r.url()) {
            md.a("Repository", repo);
        }
        if let Some(page) = page {
            md.a("Homepage", page);
        }
    }

    Ok(Some(Hover {
        range: Some(dep.range()),
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md.build(),
        }),
    }))
}

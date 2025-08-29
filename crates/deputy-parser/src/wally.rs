#![allow(unused_imports)]

use async_language_server::{lsp_types::Position, server::Document, tree_sitter::Node as TsNode};

pub use super::shared::{
    TriDependency as WallyDependency, TriDependencySpecRanges as WallyDependencySpecRanges,
    parse_dependency,
};

#[must_use]
pub fn find_all_dependencies(doc: &Document) -> Vec<TsNode<'_>> {
    super::shared::find_all_dependencies(doc, super::shared::TableNames::Wally)
}

#[must_use]
pub fn find_dependency_at(doc: &Document, pos: Position) -> Option<TsNode<'_>> {
    super::shared::find_dependency_at(doc, pos, super::shared::TableNames::Wally)
}

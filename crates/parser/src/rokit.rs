#![allow(unused_imports)]

use async_language_server::{lsp_types::Position, server::Document, tree_sitter::Node as TsNode};

pub use super::shared::{
    TriDependency as RokitDependency, TriDependencySpecRanges as RokitDependencySpecRanges,
    parse_dependency,
};

#[must_use]
pub fn find_all_dependencies(doc: &Document) -> Vec<TsNode> {
    super::shared::find_all_dependencies(doc, super::shared::TableNames::Rokit)
}

#[must_use]
pub fn find_dependency_at(doc: &Document, pos: Position) -> Option<TsNode> {
    super::shared::find_dependency_at(doc, pos, super::shared::TableNames::Rokit)
}

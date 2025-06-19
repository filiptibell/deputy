use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use async_language_server::server::Document;

use crate::parser::DependencyKind;

use super::super::query_structs::{Node, SimpleDependency};

pub fn query_rokit_toml_dependencies(_doc: &Document) -> Vec<SimpleDependency> {
    vec![]
}

use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use async_language_server::server::Document;

use super::super::query_structs::{DependencyKind, Node, SimpleDependency};

pub fn query_wally_toml_dependencies(_doc: &Document) -> Vec<SimpleDependency> {
    vec![]
}

use std::cmp::Ordering;

use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use async_language_server::server::Document;

use super::super::query_structs::{
    Dependency, DependencyKind, DependencySource, DependencySpec, Node,
};

pub fn query_package_json_dependencies(_doc: &Document) -> Vec<Dependency> {
    vec![]
}

use async_language_server::lsp_types::Range;
use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use async_language_server::server::Document;

use super::super::query_structs::{
    Dependency, DependencyKind, DependencySource, DependencySpec, Node,
};

pub fn query_cargo_toml_dependencies(_doc: &Document) -> Vec<Dependency> {
    vec![]
}

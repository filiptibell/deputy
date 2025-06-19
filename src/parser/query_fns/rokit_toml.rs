use async_language_server::server::Document;

use crate::parser::{
    query_utils::{extract_key_child, extract_key_value_pair},
    DependencyKind,
};

use super::super::query_structs::{Node, SimpleDependency};

pub fn query_rokit_toml_dependencies(doc: &Document) -> Vec<SimpleDependency> {
    let Some(root) = doc.node_at_root() else {
        return Vec::new();
    };

    let mut cursor = root.walk();
    let mut deps = Vec::new();

    for top_level in root.children(&mut cursor) {
        if top_level.kind() == "table" {
            if !matches!(extract_key_child(doc, top_level), Some((_, "tools"))) {
                continue;
            };

            let mut top_level_cursor = top_level.walk();
            for child in top_level.children(&mut top_level_cursor) {
                if let Some((key, val)) = extract_key_value_pair(doc, child) {
                    deps.push(SimpleDependency {
                        kind: DependencyKind::Default,
                        name: Node::string(&key.0, key.1),
                        spec: Node::string(&val.0, val.1),
                    });
                }
            }
        }
    }

    deps
}

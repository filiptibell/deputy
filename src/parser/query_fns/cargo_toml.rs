use async_language_server::{server::Document, tree_sitter_utils::ts_range_to_lsp_range};

use crate::parser::{
    query_utils::{extract_key_child, extract_key_value_pair, range_extend},
    DependencyKind, DependencySource, DependencySpec,
};

use super::super::query_structs::{Dependency, Node};

pub fn query_cargo_toml_dependencies(doc: &Document) -> Vec<Dependency> {
    let Some(root) = doc.node_at_root() else {
        return Vec::new();
    };

    let mut cursor = root.walk();
    let mut deps = Vec::new();

    for top_level in root.children(&mut cursor) {
        if top_level.kind() == "table" {
            let Some((_, table_name)) = extract_key_child(doc, top_level) else {
                continue;
            };

            let kind = match table_name.trim() {
                "dependencies" => DependencyKind::Default,
                "dev-dependencies" | "dev_dependencies" => DependencyKind::Dev,
                "server-dependencies" | "server_dependencies" => DependencyKind::Server,
                _ => continue,
            };

            let mut top_level_cursor = top_level.walk();
            for child in top_level.children(&mut top_level_cursor) {
                if let Some((key, val)) = extract_key_value_pair(doc, child) {
                    let spec = DependencySpec {
                        source: DependencySource::Registry,
                        version: if val.0.kind() == "string" {
                            Some(Node::string(&val.0, val.1))
                        } else {
                            None
                        },
                        features: None,
                    };
                    deps.push(Dependency::Full {
                        kind,
                        name: Node::string(&key.0, key.1),
                        range: range_extend(
                            ts_range_to_lsp_range(key.0.range()),
                            ts_range_to_lsp_range(val.0.range()),
                        ),
                        spec: Node::new(&val.0, spec),
                    });
                }
            }
        }
    }

    deps
}

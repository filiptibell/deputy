use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use crate::parser::DependencyKind;

use super::super::document::TreeSitterDocument;
use super::super::query_strings::ROKIT_TOML_DEPENDENCIES_QUERY;
use super::super::query_structs::{Node, SimpleDependency};

pub fn query_rokit_toml_dependencies(doc: &TreeSitterDocument) -> Vec<SimpleDependency> {
    let Some(query) = doc.query(ROKIT_TOML_DEPENDENCIES_QUERY) else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut tools = Vec::new();

    let mut it = cursor.matches(&query, doc.tree.root_node(), doc.contents.as_bytes());
    while let Some(m) = it.next() {
        let mut tool_name_node = None;
        let mut tool_spec_node = None;

        for capture in m.captures {
            let capture_name = query.capture_names()[capture.index as usize];
            let Ok(node_text) = capture.node.utf8_text(doc.contents.as_bytes()) else {
                continue;
            };

            match capture_name {
                "dependency_name" => {
                    tool_name_node = Some(Node::string(&capture.node, node_text));
                }
                "dependency_spec" => {
                    tool_spec_node = Some(Node::string(&capture.node, node_text));
                }
                _ => {}
            }
        }

        if let (Some(name), Some(spec)) = (tool_name_node, tool_spec_node) {
            tools.push(SimpleDependency {
                kind: DependencyKind::default(),
                name,
                spec,
            });
        }
    }

    SimpleDependency::sort_vec(&mut tools);

    tools
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use url::Url;

    use super::*;

    fn test_tools(contents: &str, expected: Vec<(&'static str, &'static str)>) {
        let path = Path::new("rokit.toml");
        let file = TreeSitterDocument::new_file(path, contents).unwrap();
        let tools = query_rokit_toml_dependencies(&file);

        assert_eq!(tools.len(), expected.len(), "mismatched number of tools");

        for (tool, (name, spec)) in tools.into_iter().zip(expected.into_iter()) {
            assert_eq!(tool.name.contents, name);
            assert_eq!(tool.spec.unquoted(), spec);
        }
    }

    #[test]
    fn test_empty() {
        test_tools("[tools]", vec![]);
    }

    #[test]
    fn test_single() {
        test_tools(
            r#"
            [tools]
            stylua = "JohnnyMorganz/StyLua@2.0.2"
            "#,
            vec![("stylua", "JohnnyMorganz/StyLua@2.0.2")],
        );
    }

    #[test]
    fn test_multiple() {
        test_tools(
            r#"
            [tools]
            stylua = "JohnnyMorganz/StyLua@2.0.2"
            wally = "UpliftGames/wally@0.3.2"
            "#,
            vec![
                ("stylua", "JohnnyMorganz/StyLua@2.0.2"),
                ("wally", "UpliftGames/wally@0.3.2"),
            ],
        );
    }
}

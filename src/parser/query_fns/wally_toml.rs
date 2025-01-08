use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use super::super::document::TreeSitterDocument;
use super::super::query_strings::WALLY_TOML_DEPENDENCIES_QUERY;
use super::super::query_structs::{DependencyKind, Node, SimpleDependency};

pub fn query_wally_toml_dependencies(doc: &TreeSitterDocument) -> Vec<SimpleDependency> {
    let Some(query) = doc.query(WALLY_TOML_DEPENDENCIES_QUERY) else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut dependencies = Vec::new();

    let mut it = cursor.matches(&query, doc.tree.root_node(), doc.contents.as_bytes());
    while let Some(m) = it.next() {
        let mut dep_kind = None;
        let mut dep_name_node = None;
        let mut dep_spec_node = None;
        let mut spec_range = None;

        for capture in m.captures {
            let capture_name = query.capture_names()[capture.index as usize];
            let Ok(node_text) = capture.node.utf8_text(doc.contents.as_bytes()) else {
                continue;
            };

            match capture_name {
                "root_name" => {
                    dep_kind = Some(match node_text {
                        "dependencies" => DependencyKind::Default,
                        "dev-dependencies" => DependencyKind::Dev,
                        "server-dependencies" => DependencyKind::Server,
                        _ => continue,
                    });
                }
                "dependency_name" => {
                    dep_name_node = Some(Node::string(&capture.node, node_text));
                }
                "dependency_spec" => {
                    dep_spec_node = Some(Node::string(&capture.node, node_text));
                    if spec_range.is_none() {
                        spec_range = Some(capture.node.parent().unwrap());
                    }
                }
                _ => {}
            }
        }

        if let (Some(dep_kind), Some(name), Some(spec)) = (dep_kind, dep_name_node, dep_spec_node) {
            dependencies.push(SimpleDependency {
                kind: dep_kind,
                name,
                spec,
            });
        }
    }

    SimpleDependency::sort_vec(&mut dependencies);

    dependencies
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use url::Url;

    use super::*;

    fn test_dependencies(
        contents: &str,
        expected: Vec<(DependencyKind, &'static str, &'static str)>,
    ) {
        let path = Path::new("wally.toml");
        let file = TreeSitterDocument::new_file(path, contents).unwrap();
        let deps = query_wally_toml_dependencies(&file);

        assert_eq!(
            deps.len(),
            expected.len(),
            "mismatched number of dependencies"
        );

        for (dep, (kind, name, spec)) in deps.into_iter().zip(expected.into_iter()) {
            assert_eq!(dep.kind, kind);
            assert_eq!(dep.name.unquoted(), name);
            assert_eq!(dep.spec.unquoted(), spec);
        }
    }

    #[test]
    fn test_empty() {
        test_dependencies("[dependencies]", vec![]);
    }

    #[test]
    fn test_single() {
        test_dependencies(
            r#"
            [dependencies]
            Fusion = "elttob/fusion@0.3.0"
            "#,
            vec![(DependencyKind::Default, "Fusion", "elttob/fusion@0.3.0")],
        );
    }

    #[test]
    fn test_multiple() {
        test_dependencies(
            r#"
            [dependencies]
            Fusion = "elttob/fusion@0.3.0"
            UILabs = "pepeeltoro41/ui-labs@2.3.0"
            "#,
            vec![
                (DependencyKind::Default, "Fusion", "elttob/fusion@0.3.0"),
                (
                    DependencyKind::Default,
                    "UILabs",
                    "pepeeltoro41/ui-labs@2.3.0",
                ),
            ],
        );
    }

    #[test]
    fn test_server_dependencies() {
        test_dependencies(
            r#"
            [server-dependencies]
            ServerPkg = "user/repo@1.0.0"
            "#,
            vec![(DependencyKind::Server, "ServerPkg", "user/repo@1.0.0")],
        );
    }

    #[test]
    fn test_dev_dependencies() {
        test_dependencies(
            r#"
            [dev-dependencies]
            TestPkg = "user/repo@1.0.0"
            "#,
            vec![(DependencyKind::Dev, "TestPkg", "user/repo@1.0.0")],
        );
    }

    #[test]
    fn test_mixed_dependencies() {
        test_dependencies(
            r#"
            [dependencies]
            Fusion = "elttob/fusion@0.3.0"

            [dev-dependencies]
            TestPkg = "user/repo@1.0.0"

            [server-dependencies]
            ServerPkg = "user/repo@1.0.0"
            "#,
            vec![
                (DependencyKind::Default, "Fusion", "elttob/fusion@0.3.0"),
                (DependencyKind::Dev, "TestPkg", "user/repo@1.0.0"),
                (DependencyKind::Server, "ServerPkg", "user/repo@1.0.0"),
            ],
        );
    }
}

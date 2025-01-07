use streaming_iterator::StreamingIterator;
use tower_lsp::lsp_types::Range;
use tree_sitter::QueryCursor;

use crate::parser::query_utils::{range_extend, range_from_node};

use super::super::document::TreeSitterDocument;
use super::super::query_strings::CARGO_TOML_DEPENDENCIES_QUERY;
use super::super::query_structs::{
    Dependency, DependencyKind, DependencySource, DependencySpec, Node,
};

pub fn query_cargo_toml_dependencies(doc: &TreeSitterDocument) -> Vec<Dependency> {
    let Some(query) = doc.query(CARGO_TOML_DEPENDENCIES_QUERY) else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut dependencies = Vec::new();

    let mut it = cursor.matches(&query, doc.tree.root_node(), doc.contents.as_bytes());
    while let Some(m) = it.next() {
        let mut dep_kind = None;
        let mut dep_range = None;
        let mut dep_name_node = None;
        let mut version_node = None;
        let mut features = Vec::new();
        let mut features_range = None;
        let mut spec_range = None::<Range>;

        for capture in m.captures {
            let capture_name = query.capture_names()[capture.index as usize];
            let Ok(node_text) = capture.node.utf8_text(doc.contents.as_bytes()) else {
                continue;
            };

            match capture_name {
                "root_name" => {
                    dep_kind = Some(match node_text {
                        "dependencies" => DependencyKind::Default,
                        "dev-dependencies" | "dev_dependencies" => DependencyKind::Dev,
                        "build-dependencies" | "build_dependencies" => DependencyKind::Build,
                        _ => continue,
                    });
                }
                "dependency_name" | "incomplete_dependency_name" => {
                    dep_name_node = Some(Node::string(&capture.node, node_text));
                }
                "version" => {
                    version_node = Some(Node::string(&capture.node, node_text));
                }
                "features_array" => {
                    if features_range.is_none() {
                        features_range = Some(&capture.node);
                        for child in capture.node.named_children(&mut capture.node.walk()) {
                            if child.kind() == "string" {
                                if let Ok(child_text) = child.utf8_text(doc.contents.as_bytes()) {
                                    features.push(Node::string(&child, child_text));
                                };
                            }
                        }
                    }
                }
                _ => {}
            }

            if matches!(
                capture_name,
                "dependency_name"
                    | "incomplete_dependency_name"
                    | "dependency_table"
                    | "dependency_full_capture"
                    | "version"
                    | "features_array"
            ) {
                let range = range_from_node(&capture.node);
                if let Some(drange) = dep_range {
                    dep_range = Some(range_extend(range, drange));
                } else {
                    dep_range = Some(range);
                }
            }

            if matches!(
                capture_name,
                "dependency_table" | "version" | "features_array"
            ) {
                let range = range_from_node(&capture.node);
                if let Some(srange) = spec_range {
                    spec_range = Some(range_extend(range, srange));
                } else {
                    spec_range = Some(range);
                }
            }
        }

        if let (Some(dep_kind), Some(range), Some(name)) = (dep_kind, dep_range, dep_name_node) {
            dependencies.push(Dependency::new_opt(
                dep_kind,
                range,
                name,
                spec_range.map(|r| {
                    Node::new_raw(
                        r,
                        DependencySpec {
                            source: DependencySource::Registry,
                            version: version_node,
                            features: features_range.map(|r| Node::new(r, features)),
                        },
                    )
                }),
            ));
        }
    }

    dependencies
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use url::Url;

    use super::*;

    fn test_dependencies(
        contents: &str,
        expected: Vec<(
            DependencyKind,
            &'static str,
            &'static str,
            Vec<&'static str>,
        )>,
    ) {
        let path = Path::new("Cargo.toml");
        let file = TreeSitterDocument::new_file(path, contents).unwrap();
        let deps = query_cargo_toml_dependencies(&file);

        assert_eq!(
            deps.len(),
            expected.len(),
            "mismatched number of dependencies!\ndependencies: {deps:#?}\nexpected: {expected:?}"
        );

        for (dep, (kind, name, version, features)) in deps.into_iter().zip(expected.into_iter()) {
            assert_eq!(dep.kind(), kind);
            assert_eq!(dep.name().contents, name);
            assert_eq!(
                dep.spec()
                    .unwrap()
                    .contents
                    .version
                    .as_ref()
                    .unwrap()
                    .unquoted(),
                version
            );
            assert_eq!(
                dep.spec()
                    .unwrap()
                    .clone()
                    .contents
                    .features
                    .into_iter()
                    .flat_map(|f| f.contents.into_iter().map(|f| f.unquoted().to_string()))
                    .collect::<Vec<_>>(),
                features
            );
        }
    }

    #[test]
    fn test_empty() {
        test_dependencies("[dependencies]", vec![]);
    }

    #[test]
    fn test_simple_version() {
        test_dependencies(
            r#"
            [dependencies]
            tokio = "1.25.0"
            "#,
            vec![(DependencyKind::Default, "tokio", "1.25.0", vec![])],
        );
    }

    #[test]
    fn test_table_version() {
        test_dependencies(
            r#"
            [dependencies]
            tokio = { version = "1.25.0" }
            "#,
            vec![(DependencyKind::Default, "tokio", "1.25.0", vec![])],
        );
    }

    #[test]
    fn test_multiple_dependencies() {
        test_dependencies(
            r#"
            [dependencies]
            tokio = "1.25.0"
            serde = { version = "1.0.160" }
            "#,
            vec![
                (DependencyKind::Default, "tokio", "1.25.0", vec![]),
                (DependencyKind::Default, "serde", "1.0.160", vec![]),
            ],
        );
    }

    #[test]
    fn test_dev_dependencies() {
        test_dependencies(
            r#"
            [dev-dependencies]
            pretty_assertions = "1.3.0"
            "#,
            vec![(DependencyKind::Dev, "pretty_assertions", "1.3.0", vec![])],
        );
    }

    #[test]
    fn test_build_dependencies() {
        test_dependencies(
            r#"
            [build-dependencies]
            cc = "1.0"
            "#,
            vec![(DependencyKind::Build, "cc", "1.0", vec![])],
        );
    }

    #[test]
    fn test_mixed_dependencies() {
        test_dependencies(
            r#"
            [dependencies]
            tokio = "1.25.0"

            [dev-dependencies]
            pretty_assertions = "1.3.0"

            [build-dependencies]
            cc = "1.0"
            "#,
            vec![
                (DependencyKind::Default, "tokio", "1.25.0", vec![]),
                (DependencyKind::Dev, "pretty_assertions", "1.3.0", vec![]),
                (DependencyKind::Build, "cc", "1.0", vec![]),
            ],
        );
    }

    #[test]
    fn test_features() {
        test_dependencies(
            r#"
            [dependencies]
            tokio = { version = "1.25.0", features = ["full", "macros"] }
            "#,
            vec![(
                DependencyKind::Default,
                "tokio",
                "1.25.0",
                vec!["full", "macros"],
            )],
        );
    }

    #[test]
    fn test_incomplete_dependency_first() {
        let contents = r#"
        [dependencies]
        incomplete
        tokio = "1.0"
        serde = "1.0"
        "#;

        let path = Path::new("Cargo.toml");
        let file = TreeSitterDocument::new_file(path, contents).unwrap();
        let deps = query_cargo_toml_dependencies(&file);

        assert_eq!(deps.len(), 3, "mismatched number of dependencies");

        let dep = deps.first().unwrap();
        assert_eq!(dep.kind(), DependencyKind::Default);
        assert_eq!(dep.name().contents, "incomplete");
        assert!(dep.spec().is_none());
    }

    #[test]
    fn test_incomplete_dependency_middle() {
        let contents = r#"
        [dependencies]
        tokio = "1.0"
        incomplete
        serde = "1.0"
        "#;

        let path = Path::new("Cargo.toml");
        let file = TreeSitterDocument::new_file(path, contents).unwrap();
        let deps = query_cargo_toml_dependencies(&file);

        assert_eq!(deps.len(), 3, "mismatched number of dependencies");

        let dep = deps.get(1).unwrap();
        assert_eq!(dep.kind(), DependencyKind::Default);
        assert_eq!(dep.name().contents, "incomplete");
        assert!(dep.spec().is_none());
    }

    #[test]
    fn test_incomplete_dependency_hanging() {
        let contents = r#"
        [dependencies]
        tokio = "1.0"
        incomplete
        "#;

        let path = Path::new("Cargo.toml");
        let file = TreeSitterDocument::new_file(path, contents).unwrap();
        let deps = query_cargo_toml_dependencies(&file);

        assert_eq!(deps.len(), 2, "mismatched number of dependencies");

        let dep = deps.get(1).unwrap();
        assert_eq!(dep.kind(), DependencyKind::Default);
        assert_eq!(dep.name().contents, "incomplete");
        assert!(dep.spec().is_none());
    }
}

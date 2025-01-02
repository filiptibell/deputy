use std::cmp::Ordering;

use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use super::super::file::TreeSitterFile;
use super::super::query_strings::PACKAGE_JSON_DEPENDENCIES_QUERY;
use super::super::query_structs::{
    Dependency, DependencyKind, DependencySource, DependencySpec, Node,
};

pub fn query_package_json_dependencies(file: &TreeSitterFile) -> Vec<Dependency> {
    let Some(query) = file.query(PACKAGE_JSON_DEPENDENCIES_QUERY) else {
        return Vec::new();
    };

    let mut cursor = QueryCursor::new();
    let mut dependencies = Vec::new();

    let mut it = cursor.matches(&query, file.tree.root_node(), file.contents.as_bytes());
    while let Some(m) = it.next() {
        let mut dep_kind = None;
        let mut dep_name_node = None;
        let mut version_node = None;
        let mut git_url = None;
        let mut path = None;
        let mut spec_range = None;

        for capture in m.captures {
            let capture_name = query.capture_names()[capture.index as usize];
            let Ok(node_text) = capture.node.utf8_text(file.contents.as_bytes()) else {
                continue;
            };

            match capture_name {
                "root_name" => {
                    dep_kind = Some(match node_text {
                        "dependencies" => DependencyKind::Default,
                        "devDependencies" => DependencyKind::Dev,
                        "peerDependencies" => DependencyKind::Peer,
                        "optionalDependencies" => DependencyKind::Optional,
                        _ => continue,
                    });
                }
                "dependency_name" => {
                    dep_name_node = Some(Node::string(&capture.node, node_text));
                }
                "value" => {
                    if node_text.starts_with("git") || node_text.ends_with(".git") {
                        git_url = Some(Node::string(&capture.node, node_text));
                    } else if node_text.starts_with("file:")
                        || node_text.starts_with("./")
                        || node_text.starts_with("../")
                    {
                        path = Some(Node::string(&capture.node, node_text));
                    } else {
                        version_node = Some(Node::string(&capture.node, node_text));
                    }
                    if spec_range.is_none() {
                        spec_range = Some(capture.node.parent().unwrap());
                    }
                }
                _ => {}
            }
        }

        if let (Some(dep_kind), Some(name), Some(spec_range)) =
            (dep_kind, dep_name_node, spec_range)
        {
            let source = if let Some(url) = git_url {
                DependencySource::Git { url }
            } else if let Some(path) = path {
                DependencySource::Path { path }
            } else {
                DependencySource::Registry
            };

            dependencies.push(Dependency {
                kind: dep_kind,
                name,
                spec: Node::new(
                    &spec_range,
                    DependencySpec {
                        source,
                        version: version_node,
                        features: None, // NPM doesn't have features
                    },
                ),
            });
        }
    }

    Dependency::sort_vec(&mut dependencies);

    dependencies
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tower_lsp::lsp_types::{Position, Range};
    use url::Url;

    use super::*;

    fn test_dependencies(
        contents: &str,
        expected: Vec<(
            DependencyKind,
            &'static str,
            Option<&'static str>,
            Option<DependencySource>,
        )>,
    ) {
        let uri = Url::from_file_path(Path::new("package.json")).unwrap();
        let file = TreeSitterFile::new(uri, contents.to_string()).unwrap();
        let deps = query_package_json_dependencies(&file);

        assert_eq!(
            deps.len(),
            expected.len(),
            "mismatched number of dependencies"
        );

        for (dep, (kind, name, version, source_opt)) in deps.into_iter().zip(expected.into_iter()) {
            assert_eq!(dep.kind, kind);
            assert_eq!(dep.name.contents, name);
            assert_eq!(
                dep.spec.contents.version.as_ref().map(|v| v.unquoted()),
                version
            );
            if let Some(source) = source_opt {
                assert_eq!(dep.spec.contents.source.contents(), source.contents());
            }
        }
    }

    fn dep_source_zeroed_node_git(s: &str) -> DependencySource {
        DependencySource::Git {
            url: Node {
                contents: s.to_string(),
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 0),
                },
            },
        }
    }

    fn dep_source_zeroed_node_path(s: &str) -> DependencySource {
        DependencySource::Path {
            path: Node {
                contents: s.to_string(),
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(0, 0),
                },
            },
        }
    }

    #[test]
    fn test_empty() {
        test_dependencies(
            r#"{
            	"dependencies": {}
            }"#,
            vec![],
        );
    }

    #[test]
    fn test_simple_version() {
        test_dependencies(
            r#"{
                "dependencies": {
                    "express": "^4.17.1"
                }
            }"#,
            vec![(DependencyKind::Default, "express", Some("^4.17.1"), None)],
        );
    }

    #[test]
    fn test_multiple_dependencies() {
        test_dependencies(
            r#"{
                "dependencies": {
                    "express": "^4.17.1",
                    "typescript": "~4.5.0"
                }
            }"#,
            vec![
                (DependencyKind::Default, "express", Some("^4.17.1"), None),
                (DependencyKind::Default, "typescript", Some("~4.5.0"), None),
            ],
        );
    }

    #[test]
    fn test_dev_dependencies() {
        test_dependencies(
            r#"{
                "devDependencies": {
                    "jest": "^27.0.0"
                }
            }"#,
            vec![(DependencyKind::Dev, "jest", Some("^27.0.0"), None)],
        );
    }

    #[test]
    fn test_peer_dependencies() {
        test_dependencies(
            r#"{
                "peerDependencies": {
                    "react": "^17.0.0"
                }
            }"#,
            vec![(DependencyKind::Peer, "react", Some("^17.0.0"), None)],
        );
    }

    #[test]
    fn test_optional_dependencies() {
        test_dependencies(
            r#"{
                "optionalDependencies": {
                    "colors": "^1.4.0"
                }
            }"#,
            vec![(DependencyKind::Optional, "colors", Some("^1.4.0"), None)],
        );
    }

    #[test]
    fn test_mixed_dependencies() {
        test_dependencies(
            r#"{
                "dependencies": {
                    "express": "^4.17.1"
                },
                "devDependencies": {
                    "jest": "^27.0.0"
                },
                "peerDependencies": {
                    "react": "^17.0.0"
                }
            }"#,
            vec![
                (DependencyKind::Default, "express", Some("^4.17.1"), None),
                (DependencyKind::Dev, "jest", Some("^27.0.0"), None),
                (DependencyKind::Peer, "react", Some("^17.0.0"), None),
            ],
        );
    }

    #[test]
    fn test_git_dependencies() {
        test_dependencies(
            r#"{
                "dependencies": {
                    "debug": "git://github.com/debug/debug.git#master",
                    "express": "git+https://github.com/expressjs/express.git"
                }
            }"#,
            vec![
                (
                    DependencyKind::Default,
                    "debug",
                    None,
                    Some(dep_source_zeroed_node_git(
                        "git://github.com/debug/debug.git#master",
                    )),
                ),
                (
                    DependencyKind::Default,
                    "express",
                    None,
                    Some(dep_source_zeroed_node_git(
                        "git+https://github.com/expressjs/express.git",
                    )),
                ),
            ],
        );
    }

    #[test]
    fn test_local_dependencies() {
        test_dependencies(
            r#"{
                "dependencies": {
                    "local-pkg": "file:../local-pkg",
                    "sibling-pkg": "file:./sibling-pkg/index.js",
                }
            }"#,
            vec![
                (
                    DependencyKind::Default,
                    "local-pkg",
                    None,
                    Some(dep_source_zeroed_node_path("file:../local-pkg")),
                ),
                (
                    DependencyKind::Default,
                    "sibling-pkg",
                    None,
                    Some(dep_source_zeroed_node_path("file:./sibling-pkg/index.js")),
                ),
            ],
        );
    }

    #[test]
    fn test_mixed_sources() {
        test_dependencies(
            r#"{
                "dependencies": {
                    "express": "^4.17.1",
                    "local-pkg": "file:../local-pkg",
                    "private-pkg": "git+ssh://git@github.com/org/repo.git"
                }
            }"#,
            vec![
                (
                    DependencyKind::Default,
                    "express",
                    Some("^4.17.1"),
                    Some(DependencySource::Registry),
                ),
                (
                    DependencyKind::Default,
                    "local-pkg",
                    None,
                    Some(dep_source_zeroed_node_path("file:../local-pkg")),
                ),
                (
                    DependencyKind::Default,
                    "private-pkg",
                    None,
                    Some(dep_source_zeroed_node_git(
                        "git+ssh://git@github.com/org/repo.git",
                    )),
                ),
            ],
        );
    }
}

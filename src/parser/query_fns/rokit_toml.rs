use std::ops::Range;

use async_language_server::{
    lsp_types::Position,
    server::Document,
    text_utils::byte_range::subrange_delimited_tri,
    tree_sitter::Node as TsNode,
    tree_sitter_utils::{find_ancestor, find_child},
};

use crate::parser::{query_utils::extract_key_value_pair, DependencyKind};

use super::super::query_structs::{Node, SimpleDependency};

pub fn query_rokit_toml_dependencies(doc: &Document) -> Vec<SimpleDependency> {
    let Some(root) = doc.node_at_root() else {
        return Vec::new();
    };

    let mut cursor = root.walk();
    let mut deps = Vec::new();

    for top_level in root.children(&mut cursor) {
        if top_level.kind() == "table" {
            let Some(key) = find_child(top_level, |c| c.kind() == "bare_key") else {
                continue;
            };
            if doc.node_text(key) != "tools" {
                continue;
            }

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

pub fn find_all_dependencies(doc: &Document) -> Vec<TsNode> {
    let Some(root) = doc.node_at_root() else {
        return Vec::new();
    };

    let mut cursor = root.walk();
    let mut deps = Vec::new();

    for top_level in root.children(&mut cursor) {
        let Some(key) = find_child(top_level, |c| c.kind() == "bare_key") else {
            continue;
        };

        if doc.node_text(key) != "tools" {
            continue;
        }

        let mut top_level_cursor = top_level.walk();
        for child in top_level.children(&mut top_level_cursor) {
            if child.kind() == "pair" {
                deps.push(child);
            }
        }
    }

    deps
}

pub fn find_nearest_dependency(doc: &Document, pos: Position) -> Option<TsNode> {
    let node = doc.node_at_position(pos)?; // either the key or value
    let pair = find_ancestor(node, |a| a.kind() == "pair")?; // tool-name = "spec"

    let table = find_ancestor(node, |a| a.kind() == "table")?;
    let key = find_child(table, |c| c.kind() == "bare_key")?;
    if doc.node_text(key) != "tools" {
        return None;
    }

    Some(pair)
}

pub fn parse_dependency<'tree>(pair: TsNode<'tree>) -> Option<RokitDependency<'tree>> {
    Some(RokitDependency {
        alias: find_child(pair, |c| c.kind() == "bare_key")?,
        spec: find_child(pair, |c| c.kind() == "string")?,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct RokitDependency<'tree> {
    pub alias: TsNode<'tree>,
    pub spec: TsNode<'tree>,
}

impl RokitDependency<'_> {
    pub fn spec_ranges(&self, doc: &Document) -> RokitDependencySpecRanges {
        let text = doc.node_text(self.spec);
        let range = self.spec.byte_range();

        let (owner, repository, version) = subrange_delimited_tri(text.as_str(), range, '/', '@');

        RokitDependencySpecRanges {
            owner,
            repository,
            version,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RokitDependencySpecRanges {
    pub owner: Option<Range<usize>>,
    pub repository: Option<Range<usize>>,
    pub version: Option<Range<usize>>,
}

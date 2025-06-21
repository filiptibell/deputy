use std::{collections::HashMap, str::FromStr};

use async_language_server::{
    lsp_types::Position,
    server::Document,
    tree_sitter::Node as TsNode,
    tree_sitter_utils::{find_ancestor, find_child, ts_range_contains_lsp_position},
};

use super::utils::{table_key_parts, unquote};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyKind {
    Dependency,
    DevDependency,
    BuildDependency,
}

impl FromStr for DependencyKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dependencies" => Ok(DependencyKind::Dependency),
            "dev-dependencies" | "dev_dependencies" => Ok(DependencyKind::DevDependency),
            "build-dependencies" | "build_dependencies" => Ok(DependencyKind::BuildDependency),
            _ => Err(()),
        }
    }
}

fn check_dependencies_table_multi(doc: &Document, node: TsNode) -> Option<DependencyKind> {
    let parts = table_key_parts(doc, node);

    let part = if parts.first().is_some_and(|p| p == "workspace") {
        if parts.len() != 2 {
            return None;
        }
        // [workspace.dependencies]
        parts.get(1).unwrap()
    } else if parts.first().is_some_and(|p| p == "target") {
        if parts.len() != 3 {
            return None;
        }
        // [target."xx-yy-zz".dependencies]
        parts.get(2).unwrap()
    } else {
        if parts.len() != 1 {
            return None;
        }
        // [dependencies]
        parts.first().unwrap()
    };

    DependencyKind::from_str(part).ok()
}

fn check_dependencies_table_single(
    doc: &Document,
    node: TsNode,
) -> Option<(DependencyKind, String)> {
    let parts = table_key_parts(doc, node);

    let (part0, part1) = if parts.first().is_some_and(|p| p == "workspace") {
        if parts.len() != 3 {
            return None;
        }
        // [workspace.dependencies.dependency-name]
        (parts.get(1).unwrap(), parts.get(2).unwrap())
    } else if parts.first().is_some_and(|p| p == "target") {
        if parts.len() != 4 {
            return None;
        }
        // [target."xx-yy-zz".dependencies.dependency-name]
        (parts.get(2).unwrap(), parts.get(3).unwrap())
    } else {
        if parts.len() != 2 {
            return None;
        }
        // [dependencies.dependency-name]
        (parts.first().unwrap(), parts.get(1).unwrap())
    };

    if let Ok(kind) = DependencyKind::from_str(part0) {
        Some((kind, part1.to_string()))
    } else {
        None
    }
}

#[must_use]
pub fn find_all_dependencies(doc: &Document) -> Vec<TsNode> {
    let Some(root) = doc.node_at_root() else {
        return Vec::new();
    };

    let mut cursor = root.walk();
    let mut deps = Vec::new();

    for top_level in root.children(&mut cursor) {
        if check_dependencies_table_multi(doc, top_level).is_some() {
            // [dependencies] or [workspace.dependencies] etc
            let mut top_level_cursor = top_level.walk();
            for child in top_level.children(&mut top_level_cursor) {
                if child.kind() == "pair" {
                    deps.push(child);
                }
            }
        } else if check_dependencies_table_single(doc, top_level).is_some() {
            // [dependencies.name] or [workspace.dependencies.name] etc
            deps.push(top_level);
        }
    }

    deps
}

#[must_use]
pub fn find_dependency_at(doc: &Document, pos: Position) -> Option<TsNode> {
    let node = doc.node_at_position(pos)?; // either the key or value

    if let Some(table) = find_ancestor(node, |a| check_dependencies_table_single(doc, a).is_some())
    {
        // [dependencies.name] or [workspace.dependencies.name] etc
        Some(table)
    } else if let Some(table) =
        find_ancestor(node, |a| check_dependencies_table_multi(doc, a).is_some())
    {
        // dependency-name = "spec" or dependency-name = { version = "a.b.c" }
        find_child(table, |c| {
            c.kind() == "pair" && ts_range_contains_lsp_position(c.range(), pos)
        })
    } else {
        None
    }
}

#[must_use]
pub fn parse_dependency<'tree>(
    doc: &Document,
    pair_or_table: TsNode<'tree>,
) -> Option<CargoDependency<'tree>> {
    if pair_or_table.kind() == "pair" {
        let mut name = pair_or_table.named_child(0)?;
        let value = pair_or_table.named_child(1)?;

        // version is either `name = "version"` or `name = { version = "version" }`
        let mut version = None;
        let mut features = None;
        let mut package = None;
        if value.kind() == "string" {
            version = Some(value);
        } else if value.kind() == "inline_table" {
            let mut pairs = HashMap::new();
            let mut cursor = value.walk();
            for child in value.children(&mut cursor) {
                if child.kind() == "pair" {
                    let key = child.named_child(0)?;
                    let value = child.named_child(1)?;
                    pairs.insert(doc.node_text(key), value);
                }
            }
            version = pairs.remove("version");
            features = pairs.remove("features");
            package = pairs.remove("package");
        }

        // aliased_serde = { package = "serde" }
        if let Some(package) = package {
            name = package;
        }

        Some(CargoDependency {
            name,
            version: version?,
            features,
        })
    } else if pair_or_table.kind() == "table" {
        // alias is last part in [dependencies."abcdef"."ghijkl".name]
        let key = pair_or_table.named_child(0)?;
        let mut name = key.named_children(&mut key.walk()).last()?;

        let mut pairs = HashMap::new();
        let mut cursor = pair_or_table.walk();
        for child in pair_or_table.children(&mut cursor) {
            if child.kind() == "pair" {
                let key = child.named_child(0)?;
                let value = child.named_child(1)?;
                pairs.insert(doc.node_text(key), value);
            }
        }

        let version = pairs.remove("version");
        let features = pairs.remove("features");
        let package = pairs.remove("package");

        // [dependencies.aliased_serde]
        // package = "serde"
        if let Some(package) = package {
            name = package;
        }

        Some(CargoDependency {
            name,
            version: version?,
            features,
        })
    } else {
        None
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CargoDependency<'tree> {
    pub name: TsNode<'tree>,
    pub version: TsNode<'tree>,
    pub features: Option<TsNode<'tree>>,
}

impl CargoDependency<'_> {
    #[must_use]
    pub fn text(&self, doc: &Document) -> (String, String) {
        let name = doc.node_text(self.name);
        let version = doc.node_text(self.version);
        (unquote(name), unquote(version))
    }

    #[must_use]
    pub fn feature_nodes(&self) -> Vec<TsNode<'_>> {
        let mut nodes = Vec::new();
        if let Some(features) = self.features {
            let mut cursor = features.walk();
            for child in features.children(&mut cursor) {
                if child.kind() == "string" {
                    nodes.push(child);
                }
            }
        }
        nodes
    }
}

use std::str::FromStr;

use async_language_server::{
    lsp_types::Position,
    server::Document,
    tree_sitter::Node as TsNode,
    tree_sitter_utils::{find_ancestor, find_child},
};

use super::utils::table_key_parts;

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

pub fn find_dependency_at(doc: &Document, pos: Position) -> Option<TsNode> {
    let node = doc.node_at_position(pos)?; // either the key or value

    if let Some(pair) = find_ancestor(node, |a| a.kind() == "pair") {
        // dependency-name = "spec" or dependency-name = { version = "a.b.c" }
        let table = find_ancestor(node, |a| a.kind() == "table")?;
        check_dependencies_table_multi(doc, table)?;
        Some(pair)
    } else if let Some(table) = find_ancestor(node, |a| a.kind() == "table") {
        // [dependencies.name] or [workspace.dependencies.name] etc
        check_dependencies_table_single(doc, node)?;
        Some(table)
    } else {
        None
    }
}

pub fn parse_dependency<'tree>(
    doc: &Document,
    pair_or_table: TsNode<'tree>,
) -> Option<CargoDependency<'tree>> {
    if pair_or_table.kind() == "pair" {
        let name = pair_or_table.named_child(0)?;
        let value = pair_or_table.named_child(1)?;

        // version is either `name = "version"` or `name = { version = "version" }`
        let version = if value.kind() == "string" {
            value
        } else if value.kind() == "inline_table" {
            let version_pair = find_child(value, |pair| {
                let is_pair = pair.kind() == "pair";
                let is_version = pair
                    .named_child(0)
                    .is_some_and(|c| doc.node_text(c) == "version");
                is_pair && is_version
            })?;
            version_pair.named_child(1)?
        } else {
            return None;
        };

        Some(CargoDependency { name, version })
    } else if pair_or_table.kind() == "table" {
        // alias is last part in [dependencies."abcdef"."ghijkl".name]
        let mut cursor = pair_or_table.walk();
        let name = pair_or_table.named_children(&mut cursor).last()?;

        // version is always `version = "version"`
        let version_pair = find_child(pair_or_table, |pair| {
            let is_pair = pair.kind() == "pair";
            let is_version = pair
                .named_child(0)
                .is_some_and(|c| doc.node_text(c) == "version");
            is_pair && is_version
        })?;
        let version = version_pair.named_child(1)?;

        Some(CargoDependency { name, version })
    } else {
        None
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CargoDependency<'tree> {
    pub name: TsNode<'tree>,
    pub version: TsNode<'tree>,
}

use std::str::FromStr;

use async_language_server::{
    lsp_types::Position, server::Document, tree_sitter::Node as TsNode,
    tree_sitter_utils::find_ancestor,
};

use super::utils::unquote;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyKind {
    Dependency,
    DevDependency,
    PeerDependency,
    OptionalDependency,
}

impl FromStr for DependencyKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dependencies" => Ok(DependencyKind::Dependency),
            "devDependencies" => Ok(DependencyKind::DevDependency),
            "peerDependencies" => Ok(DependencyKind::PeerDependency),
            "optionalDependencies" => Ok(DependencyKind::OptionalDependency),
            _ => Err(()),
        }
    }
}

#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn find_all_dependencies(doc: &Document) -> Vec<TsNode> {
    // package.json should always have a single json object at root
    let Some(root) = doc.node_at_root() else {
        return Vec::new();
    };
    let Some(root) = root.named_child(0) else {
        return Vec::new();
    };

    let mut cursor = root.walk();
    let mut deps = Vec::new();

    for top_level in root.children(&mut cursor) {
        if top_level.kind() == "pair" {
            let key = top_level.child_by_field_name("key").expect("valid pair");
            let val = top_level.child_by_field_name("value").expect("valid pair");

            let key_str = unquote(doc.node_text(key));
            let Ok(_kind) = DependencyKind::from_str(&key_str) else {
                continue;
            };

            let mut val_cursor = val.walk();
            for dependency in val.children(&mut val_cursor) {
                if dependency.kind() == "pair" {
                    deps.push(dependency);
                }
            }
        }
    }

    deps
}

#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn find_dependency_at(doc: &Document, pos: Position) -> Option<TsNode> {
    let node = doc.node_at_position(pos)?; // either the key or value
    let pair = find_ancestor(node, |a| a.kind() == "pair")?; // "package": "spec"

    let deps_obj = find_ancestor(pair, |a| a.kind() == "object")?;
    let deps_pair = find_ancestor(deps_obj, |a| a.kind() == "pair")?;

    let key = deps_pair.child_by_field_name("key").expect("valid pair");
    let key_str = unquote(doc.node_text(key));
    DependencyKind::from_str(&key_str).ok()?;

    Some(pair)
}

#[must_use]
pub fn parse_dependency(pair: TsNode) -> Option<NpmDependency> {
    Some(NpmDependency {
        name: pair.child_by_field_name("key")?,
        spec: pair.child_by_field_name("value")?,
    })
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct NpmDependency<'tree> {
    pub name: TsNode<'tree>,
    pub spec: TsNode<'tree>,
}

impl NpmDependency<'_> {
    #[must_use]
    pub fn text(&self, doc: &Document) -> (String, String) {
        let name = doc.node_text(self.name);
        let spec = doc.node_text(self.spec);
        (unquote(name), unquote(spec))
    }
}

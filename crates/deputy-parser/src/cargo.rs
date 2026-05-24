use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use async_language_server::{
    lsp_types::Position,
    server::Document,
    tree_sitter::{Node as TsNode, Parser},
    tree_sitter_utils::{find_ancestor, find_child, ts_range_contains_lsp_position},
};

use crate::TOML_LANGUAGE;

use super::utils::{key_part_nodes, key_parts_with, unquote};

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
    check_dependencies_table_multi_inner(node, &|node| doc.node_text(node))
}

fn check_dependencies_table_multi_inner<F>(node: TsNode, node_text: &F) -> Option<DependencyKind>
where
    F: Fn(TsNode) -> String,
{
    let parts = table_key_parts_inner(node, node_text);

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
    check_dependencies_table_single_inner(node, &|node| doc.node_text(node))
}

fn check_dependencies_table_single_inner<F>(
    node: TsNode,
    node_text: &F,
) -> Option<(DependencyKind, String)>
where
    F: Fn(TsNode) -> String,
{
    let parts = table_key_parts_inner(node, node_text);

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
        Some((kind, part1.clone()))
    } else {
        None
    }
}

fn table_key_parts_inner<F>(node: TsNode, node_text: &F) -> Vec<String>
where
    F: Fn(TsNode) -> String,
{
    if node.kind() == "table"
        && let Some(key) = node.named_child(0)
    {
        key_parts_with(key, node_text)
    } else {
        Vec::new()
    }
}

#[must_use]
pub fn manifest_has_workspace(text: &str) -> bool {
    let Some(tree) = parse_toml(text) else {
        return false;
    };

    manifest_has_workspace_inner(Some(tree.root_node()), &text_fn(text))
}

fn manifest_has_workspace_inner<F>(root: Option<TsNode>, node_text: &F) -> bool
where
    F: Fn(TsNode) -> String,
{
    let Some(root) = root else { return false };

    let mut cursor = root.walk();
    root.children(&mut cursor).any(|top_level| {
        table_key_parts_inner(top_level, node_text)
            .first()
            .is_some_and(|part| part == "workspace")
    })
}

#[must_use]
pub fn workspace_dependency_names_from_text(text: &str) -> Vec<String> {
    let Some(tree) = parse_toml(text) else {
        return Vec::new();
    };

    workspace_dependency_names_inner(Some(tree.root_node()), &text_fn(text))
}

fn workspace_dependency_names_inner<F>(root: Option<TsNode<'_>>, node_text: &F) -> Vec<String>
where
    F: Fn(TsNode) -> String,
{
    let Some(root) = root else { return Vec::new() };

    let mut cursor = root.walk();
    let mut names = HashSet::new();

    for top_level in root.children(&mut cursor) {
        let parts = table_key_parts_inner(top_level, node_text);
        if parts.len() == 2 && parts[0] == "workspace" && parts[1] == "dependencies" {
            let mut top_level_cursor = top_level.walk();
            for child in top_level.children(&mut top_level_cursor) {
                if child.kind() == "pair"
                    && let Some(name) = workspace_dependency_name(child, node_text)
                {
                    names.insert(name);
                }
            }
        } else if parts.len() == 3 && parts[0] == "workspace" && parts[1] == "dependencies" {
            names.insert(parts[2].clone());
        }
    }

    let mut names = names.into_iter().collect::<Vec<_>>();
    names.sort();
    names
}

fn workspace_dependency_name<F>(pair: TsNode, node_text: &F) -> Option<String>
where
    F: Fn(TsNode) -> String,
{
    if let Some(name) = dotted_dependency_name(pair, node_text) {
        return Some(name);
    }

    let key = pair.named_child(0)?;
    let parts = key_parts_with(key, node_text);
    if parts.len() == 1 {
        parts.into_iter().next()
    } else {
        None
    }
}

fn parse_toml(text: &str) -> Option<async_language_server::tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser.set_language(&TOML_LANGUAGE.into()).ok()?;
    parser.parse(text, None)
}

fn text_fn(text: &str) -> impl Fn(TsNode<'_>) -> String + '_ {
    move |node| {
        node.utf8_text(text.as_bytes())
            .expect("node text is valid utf-8")
            .to_string()
    }
}

#[must_use]
pub fn find_all_dependencies(doc: &Document) -> Vec<TsNode<'_>> {
    find_all_dependencies_inner(doc.node_at_root(), &|node| doc.node_text(node))
}

fn find_all_dependencies_inner<'tree, F>(
    root: Option<TsNode<'tree>>,
    node_text: &F,
) -> Vec<TsNode<'tree>>
where
    F: Fn(TsNode) -> String,
{
    let Some(root) = root else { return Vec::new() };

    let mut cursor = root.walk();
    let mut deps = Vec::new();

    for top_level in root.children(&mut cursor) {
        if check_dependencies_table_multi_inner(top_level, node_text).is_some() {
            // [dependencies] or [workspace.dependencies] etc
            let mut top_level_cursor = top_level.walk();
            let mut dotted_deps = HashSet::new();
            for child in top_level.children(&mut top_level_cursor) {
                if child.kind() == "pair" {
                    if let Some(dep_name) = dotted_dependency_name(child, node_text) {
                        if dotted_deps.insert(dep_name) {
                            deps.push(child);
                        }
                    } else {
                        deps.push(child);
                    }
                }
            }
        } else if check_dependencies_table_single_inner(top_level, node_text).is_some() {
            // [dependencies.name] or [workspace.dependencies.name] etc
            deps.push(top_level);
        }
    }

    deps
}

#[must_use]
pub fn find_dependency_at(doc: &Document, pos: Position) -> Option<TsNode<'_>> {
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
    parse_dependency_inner(pair_or_table, &|node| doc.node_text(node))
}

fn parse_dependency_inner<'tree, F>(
    pair_or_table: TsNode<'tree>,
    node_text: &F,
) -> Option<CargoDependency<'tree>>
where
    F: Fn(TsNode) -> String,
{
    if pair_or_table.kind() == "pair" {
        if let Some(dep) = parse_dotted_dependency(pair_or_table, node_text) {
            return Some(dep);
        }

        let mut name = pair_or_table.named_child(0)?;
        let value = pair_or_table.named_child(1)?;

        // version is either `name = "version"` or `name = { version = "version" }`
        let mut version = None;
        let mut features = None;
        let mut package = None;
        let mut path = None;
        let mut git = None;
        let mut workspace = None;
        if value.kind() == "string" {
            version = Some(value);
        } else if value.kind() == "inline_table" {
            let mut pairs = HashMap::new();
            let mut cursor = value.walk();
            for child in value.children(&mut cursor) {
                if child.kind() == "pair" {
                    let key = child.named_child(0)?;
                    let value = child.named_child(1)?;
                    let parts = key_parts_with(key, node_text);
                    if parts.len() == 1 {
                        pairs.insert(parts[0].clone(), value);
                    }
                }
            }
            version = pairs.remove("version");
            features = pairs.remove("features");
            package = pairs.remove("package");
            path = pairs.remove("path");
            git = pairs.remove("git");
            workspace = pairs
                .remove("workspace")
                .filter(|v| is_true(&node_text(*v)));
        }

        // aliased_serde = { package = "serde" }
        if let Some(package) = package {
            name = package;
        }

        if version.is_none() && path.is_none() && git.is_none() && workspace.is_none() {
            return None; // Not a valid package
        }

        Some(CargoDependency {
            name,
            version,
            features,
            path,
            git,
            workspace,
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
                let parts = key_parts_with(key, node_text);
                if parts.len() == 1 {
                    pairs.insert(parts[0].clone(), value);
                }
            }
        }

        let version = pairs.remove("version");
        let features = pairs.remove("features");
        let package = pairs.remove("package");
        let path = pairs.remove("path");
        let git = pairs.remove("git");
        let workspace = pairs
            .remove("workspace")
            .filter(|v| is_true(&node_text(*v)));

        // [dependencies.aliased_serde]
        // package = "serde"
        if let Some(package) = package {
            name = package;
        }

        if version.is_none() && path.is_none() && git.is_none() && workspace.is_none() {
            return None; // Not a valid package
        }

        Some(CargoDependency {
            name,
            version,
            features,
            path,
            git,
            workspace,
        })
    } else {
        None
    }
}

fn parse_dotted_dependency<'tree, F>(
    pair: TsNode<'tree>,
    node_text: &F,
) -> Option<CargoDependency<'tree>>
where
    F: Fn(TsNode) -> String,
{
    let current = dotted_dependency_field(pair, node_text)?;
    let dep_name = unquote(node_text(current.name));

    let mut name = current.name;
    let mut version = None;
    let mut features = None;
    let mut package = None;
    let mut path = None;
    let mut git = None;
    let mut workspace = None;

    if let Some(table) = pair.parent().filter(|p| p.kind() == "table") {
        let mut cursor = table.walk();
        for child in table.children(&mut cursor) {
            if child.kind() != "pair" {
                continue;
            }
            let Some(field) = dotted_dependency_field(child, node_text) else {
                continue;
            };
            if unquote(node_text(field.name)) != dep_name {
                continue;
            }
            match field.key.as_str() {
                "version" => version = Some(field.value),
                "features" => features = Some(field.value),
                "package" => package = Some(field.value),
                "path" => path = Some(field.value),
                "git" => git = Some(field.value),
                "workspace" if is_true(&node_text(field.value)) => workspace = Some(field.value),
                _ => {}
            }
        }
    } else {
        match current.key.as_str() {
            "version" => version = Some(current.value),
            "features" => features = Some(current.value),
            "package" => package = Some(current.value),
            "path" => path = Some(current.value),
            "git" => git = Some(current.value),
            "workspace" if is_true(&node_text(current.value)) => workspace = Some(current.value),
            _ => {}
        }
    }

    // aliased_serde.package = "serde"
    if let Some(package) = package {
        name = package;
    }

    if version.is_none() && path.is_none() && git.is_none() && workspace.is_none() {
        return None; // Not a valid package
    }

    Some(CargoDependency {
        name,
        version,
        features,
        path,
        git,
        workspace,
    })
}

fn dotted_dependency_name<F>(pair: TsNode, node_text: &F) -> Option<String>
where
    F: Fn(TsNode) -> String,
{
    dotted_dependency_field(pair, node_text).map(|field| unquote(node_text(field.name)))
}

fn dotted_dependency_field<'tree, F>(
    pair: TsNode<'tree>,
    node_text: &F,
) -> Option<DottedDependencyField<'tree>>
where
    F: Fn(TsNode) -> String,
{
    let key = pair.named_child(0)?;
    let value = pair.named_child(1)?;
    let parts = key_parts_with(key, node_text);
    if parts.len() != 2 || !is_dependency_field(&parts[1]) {
        return None;
    }
    Some(DottedDependencyField {
        name: key_part_nodes(key).first().copied()?,
        key: parts[1].clone(),
        value,
    })
}

fn is_dependency_field(field: &str) -> bool {
    matches!(
        field,
        "version" | "features" | "package" | "path" | "git" | "workspace"
    )
}

fn is_true(value: &str) -> bool {
    value == "true"
}

#[derive(Debug, Clone)]
struct DottedDependencyField<'tree> {
    name: TsNode<'tree>,
    value: TsNode<'tree>,
    key: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CargoDependency<'tree> {
    pub name: TsNode<'tree>,
    pub version: Option<TsNode<'tree>>,
    pub features: Option<TsNode<'tree>>,
    pub path: Option<TsNode<'tree>>,
    pub git: Option<TsNode<'tree>>,
    pub workspace: Option<TsNode<'tree>>,
}

impl CargoDependency<'_> {
    #[must_use]
    pub fn text(&self, doc: &Document) -> (String, Option<String>) {
        let name = doc.node_text(self.name);
        let version = self.version.map(|v| unquote(doc.node_text(v)));
        (unquote(name), version)
    }

    #[must_use]
    pub fn path_text(&self, doc: &Document) -> Option<String> {
        self.path.map(|p| unquote(doc.node_text(p)))
    }

    #[must_use]
    pub fn git_text(&self, doc: &Document) -> Option<String> {
        self.git.map(|g| unquote(doc.node_text(g)))
    }

    #[must_use]
    pub fn is_workspace(&self) -> bool {
        self.workspace.is_some()
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

#[cfg(test)]
mod tests {
    use async_language_server::tree_sitter::{Node, Tree};

    use super::*;

    fn parse(text: &str) -> Tree {
        parse_toml(text).expect("toml parses")
    }

    fn node_text(text: &str, node: Node<'_>) -> String {
        node.utf8_text(text.as_bytes())
            .expect("node text is valid utf-8")
            .to_string()
    }

    fn text_fn(text: &str) -> impl Fn(Node<'_>) -> String + '_ {
        move |node| node_text(text, node)
    }

    #[test]
    fn finds_workspace_dependency_names() {
        let text = r#"
[workspace]

[workspace.dependencies]
serde = "1.0.0"
tokio.version = "1.0.0"
"quoted-name" = { version = "1.0.0" }

[workspace.dependencies.anyhow]
version = "1.0.0"
"#;

        let names = workspace_dependency_names_from_text(text);
        assert_eq!(names, vec!["anyhow", "quoted-name", "serde", "tokio"]);
    }

    #[test]
    fn detects_workspace_manifest() {
        assert!(manifest_has_workspace(
            r"
[workspace]
members = []
"
        ));
        assert!(!manifest_has_workspace(
            r#"
[package]
name = "member"
"#
        ));
    }

    #[test]
    fn parses_path_dependency_from_dotted_key() {
        let text = r#"
[dependencies]
your-other-package.path = "path/to/package-root"
"#;

        let tree = parse(text);
        let nodes = find_all_dependencies_inner(Some(tree.root_node()), &text_fn(text));

        assert_eq!(nodes.len(), 1);

        let dep = parse_dependency_inner(nodes[0], &text_fn(text)).unwrap();
        assert_eq!(unquote(node_text(text, dep.name)), "your-other-package");
        assert!(dep.version.is_none());
        assert_eq!(
            dep.path
                .map(|node| unquote(node_text(text, node)))
                .as_deref(),
            Some("path/to/package-root")
        );
    }

    #[test]
    fn parses_workspace_dependency_from_inline_table() {
        let text = r#"
[dependencies]
serde = { workspace = true, features = ["derive"] }
"#;

        let tree = parse(text);
        let nodes = find_all_dependencies_inner(Some(tree.root_node()), &text_fn(text));

        assert_eq!(nodes.len(), 1);

        let dep = parse_dependency_inner(nodes[0], &text_fn(text)).unwrap();
        assert_eq!(unquote(node_text(text, dep.name)), "serde");
        assert!(dep.version.is_none());
        assert!(dep.is_workspace());

        let features = dep
            .feature_nodes()
            .into_iter()
            .map(|node| unquote(node_text(text, node)))
            .collect::<Vec<_>>();
        assert_eq!(features, vec!["derive"]);
    }

    #[test]
    fn parses_workspace_dependency_from_table() {
        let text = r#"
[dependencies.serde]
workspace = true
features = ["derive"]
"#;

        let tree = parse(text);
        let nodes = find_all_dependencies_inner(Some(tree.root_node()), &text_fn(text));

        assert_eq!(nodes.len(), 1);

        let dep = parse_dependency_inner(nodes[0], &text_fn(text)).unwrap();
        assert_eq!(unquote(node_text(text, dep.name)), "serde");
        assert!(dep.version.is_none());
        assert!(dep.is_workspace());
    }

    #[test]
    fn parses_workspace_dependency_from_dotted_key() {
        let text = r#"
[dependencies]
serde.workspace = true
serde.features = ["derive"]
"#;

        let tree = parse(text);
        let nodes = find_all_dependencies_inner(Some(tree.root_node()), &text_fn(text));

        assert_eq!(nodes.len(), 1);

        let dep = parse_dependency_inner(nodes[0], &text_fn(text)).unwrap();
        assert_eq!(unquote(node_text(text, dep.name)), "serde");
        assert!(dep.version.is_none());
        assert!(dep.is_workspace());

        let features = dep
            .feature_nodes()
            .into_iter()
            .map(|node| unquote(node_text(text, node)))
            .collect::<Vec<_>>();
        assert_eq!(features, vec!["derive"]);
    }

    #[test]
    fn groups_dotted_dependency_fields() {
        let text = r#"
[dependencies]
serde.version = "1.0.0"
serde.features = ["derive"]
"#;

        let tree = parse(text);
        let nodes = find_all_dependencies_inner(Some(tree.root_node()), &text_fn(text));

        assert_eq!(nodes.len(), 1);

        let dep = parse_dependency_inner(nodes[0], &text_fn(text)).unwrap();
        assert_eq!(unquote(node_text(text, dep.name)), "serde");
        assert_eq!(
            dep.version
                .map(|node| unquote(node_text(text, node)))
                .as_deref(),
            Some("1.0.0")
        );

        let features = dep
            .feature_nodes()
            .into_iter()
            .map(|node| unquote(node_text(text, node)))
            .collect::<Vec<_>>();
        assert_eq!(features, vec!["derive"]);
    }

    #[test]
    fn parses_renamed_dependency_from_dotted_key() {
        let text = r#"
[dependencies]
aliased_serde.package = "serde"
aliased_serde.version = "1.0.0"
"#;

        let tree = parse(text);
        let nodes = find_all_dependencies_inner(Some(tree.root_node()), &text_fn(text));

        assert_eq!(nodes.len(), 1);

        let dep = parse_dependency_inner(nodes[0], &text_fn(text)).unwrap();
        assert_eq!(unquote(node_text(text, dep.name)), "serde");
        assert_eq!(
            dep.version
                .map(|node| unquote(node_text(text, node)))
                .as_deref(),
            Some("1.0.0")
        );
    }
}

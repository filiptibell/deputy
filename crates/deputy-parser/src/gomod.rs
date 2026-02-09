use async_language_server::{
    lsp_types::Position, server::Document, tree_sitter::Node as TsNode,
    tree_sitter_utils::find_ancestor,
};

fn is_require_directive(kind: &str) -> bool {
    kind == "require_directive_single" || kind == "require_directive_multi"
}

#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn find_all_dependencies(doc: &Document) -> Vec<TsNode<'_>> {
    let Some(root) = doc.node_at_root() else {
        return Vec::new();
    };

    let mut cursor = root.walk();
    let mut deps = Vec::new();

    for top_level in root.children(&mut cursor) {
        if is_require_directive(top_level.kind()) {
            let mut child_cursor = top_level.walk();
            for child in top_level.named_children(&mut child_cursor) {
                if child.kind() == "require_spec" {
                    deps.push(child);
                }
            }
        }
    }

    deps
}

#[must_use]
pub fn find_dependency_at(doc: &Document, pos: Position) -> Option<TsNode<'_>> {
    let node = doc.node_at_position(pos)?;

    // Primary: well-formed require_spec with path and version
    if let Some(spec) = find_ancestor(node, |a| a.kind() == "require_spec")
        && find_ancestor(spec, |a| is_require_directive(a.kind())).is_some()
    {
        return Some(spec);
    }

    // Fallback: inside a require directive but no valid require_spec
    // (user is mid-typing, tree-sitter produced an ERROR node)
    if let Some(module_path) = find_ancestor(node, |a| a.kind() == "module_path")
        && find_ancestor(module_path, |a| is_require_directive(a.kind())).is_some()
    {
        return Some(module_path);
    }

    // Last resort: we're on a bare identifier inside a require directive
    if find_ancestor(node, |a| is_require_directive(a.kind())).is_some() {
        return Some(node);
    }

    None
}

#[must_use]
pub fn parse_dependency(node: TsNode) -> Option<GoModDependency> {
    if node.kind() == "require_spec" {
        // Full spec: path + version
        Some(GoModDependency {
            path: node.child_by_field_name("path")?,
            version: node.child_by_field_name("version"),
        })
    } else if node.kind() == "module_path" {
        // Partial: just the path, no version yet
        Some(GoModDependency {
            path: node,
            version: None,
        })
    } else {
        // Bare identifier node inside a require directive
        Some(GoModDependency {
            path: node,
            version: None,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct GoModDependency<'tree> {
    pub path: TsNode<'tree>,
    pub version: Option<TsNode<'tree>>,
}

impl GoModDependency<'_> {
    #[must_use]
    pub fn text(&self, doc: &Document) -> (String, Option<String>) {
        let path = doc.node_text(self.path);
        let version = self.version.map(|v| doc.node_text(v));
        (path, version)
    }
}

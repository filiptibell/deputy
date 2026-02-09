use async_language_server::{
    lsp_types::Position,
    server::Document,
    text_utils::RangeExt,
    tree_sitter::{Node as TsNode, Point as TsPoint, Range as TsRange},
    tree_sitter_utils::find_ancestor,
};

use super::utils::{table_key_parts, unquote};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyKind {
    Dependency,
    OptionalDependency,
    DependencyGroup,
    BuildRequirement,
}

fn check_dependencies(doc: &Document, table: TsNode, key: &str) -> Option<DependencyKind> {
    let parts = table_key_parts(doc, table);

    if parts.first().is_some_and(|p| p == "project") {
        if parts.len() == 1 && key == "dependencies" {
            return Some(DependencyKind::Dependency);
        }
        if parts.len() == 2
            && matches!(
                parts[1].as_str(),
                "optional-dependencies" | "optional_dependencies"
            )
        {
            return Some(DependencyKind::OptionalDependency);
        }
    } else if parts.len() == 1 {
        if matches!(parts[0].as_str(), "dependency-groups" | "dependency_groups") {
            return Some(DependencyKind::DependencyGroup);
        }
        if matches!(parts[0].as_str(), "build-system" | "build_system") && key == "requires" {
            return Some(DependencyKind::BuildRequirement);
        }
    }

    None
}

#[must_use]
pub fn find_all_dependencies(doc: &Document) -> Vec<TsNode<'_>> {
    let Some(root) = doc.node_at_root() else {
        return Vec::new();
    };

    let mut cursor = root.walk();
    let mut deps = Vec::new();

    for top_level in root.children(&mut cursor) {
        let mut table_cursor = top_level.walk();
        for child in top_level.children(&mut table_cursor) {
            if child.kind() != "pair" {
                continue;
            }

            let Some(key) = child.named_child(0) else {
                continue;
            };
            let Some(value) = child.named_child(1) else {
                continue;
            };

            let key_text = unquote(doc.node_text(key));
            if check_dependencies(doc, top_level, &key_text).is_none() {
                continue;
            }

            if value.kind() != "array" {
                continue;
            }

            let mut array_cursor = value.walk();
            for element in value.children(&mut array_cursor) {
                if element.kind() == "string" {
                    deps.push(element);
                }
            }
        }
    }

    deps
}

#[must_use]
pub fn find_dependency_at(doc: &Document, pos: Position) -> Option<TsNode<'_>> {
    let node = doc.node_at_position(pos)?;

    let string_node = if node.kind() == "string" {
        node
    } else {
        find_ancestor(node, |a| a.kind() == "string")?
    };

    let array = find_ancestor(string_node, |a| a.kind() == "array")?;
    let pair = find_ancestor(array, |a| a.kind() == "pair")?;
    let table = find_ancestor(pair, |a| a.kind() == "table")?;

    let key = pair.named_child(0)?;
    let key_text = unquote(doc.node_text(key));
    check_dependencies(doc, table, &key_text)?;

    Some(string_node)
}

#[must_use]
pub fn parse_dependency(string_node: TsNode) -> Option<PyProjectDependency> {
    if string_node.kind() == "string" {
        Some(PyProjectDependency {
            spec_node: string_node,
        })
    } else {
        None
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct PyProjectDependency<'tree> {
    pub spec_node: TsNode<'tree>,
}

impl PyProjectDependency<'_> {
    #[must_use]
    pub fn spec_ranges(&self, doc: &Document) -> PyProjectDependencySpecRanges {
        let text = unquote(doc.node_text(self.spec_node));
        let range = self.spec_node.range().shrink(1, 1);
        split_pep508_ranges(&text, range)
    }

    #[must_use]
    pub fn text(&self, doc: &Document) -> (Option<String>, Option<String>) {
        let ranges = self.spec_ranges(doc);
        let txt = doc.text();

        let name = ranges
            .name
            .as_ref()
            .map(|r| r.start_byte..r.end_byte)
            .and_then(|r| txt.byte_slice(r).as_str())
            .map(|s| s.trim().to_string());

        let version = ranges
            .version
            .as_ref()
            .map(|r| r.start_byte..r.end_byte)
            .and_then(|r| txt.byte_slice(r).as_str())
            .map(|s| s.trim().to_string());

        (name, version)
    }

    #[must_use]
    pub fn raw_spec(&self, doc: &Document) -> String {
        unquote(doc.node_text(self.spec_node))
    }
}

#[derive(Debug, Clone)]
pub struct PyProjectDependencySpecRanges {
    pub name: Option<TsRange>,
    pub version: Option<TsRange>,
    pub extras: Option<TsRange>,
}

fn split_pep508_ranges(text: &str, base: TsRange) -> PyProjectDependencySpecRanges {
    // Walk the text once, recording delimiter positions as byte offsets.
    // All PEP 508 delimiters are single-byte ASCII, and TsPoint columns
    // are byte offsets, so we can use byte positions directly as TsPoints.

    let mut end = 0; // effective end (before `;` and trailing whitespace)
    let mut extras_start = None; // byte offset of `[`
    let mut extras_end = None; // byte offset past `]`
    let mut version_start = None; // byte offset of first `><=~!` after extras
    let mut in_extras = false;

    for (i, ch) in text.char_indices() {
        let next = i + ch.len_utf8();

        // Environment marker — stop here
        if ch == ';' {
            break;
        }

        if ch == '[' && extras_start.is_none() {
            extras_start = Some(i);
            in_extras = true;
        } else if ch == ']' && in_extras {
            extras_end = Some(next);
            in_extras = false;
        } else if !in_extras
            && version_start.is_none()
            && (extras_end.is_some() || extras_start.is_none())
            && matches!(ch, '>' | '<' | '=' | '~' | '!')
        {
            version_start = Some(i);
        }

        // Track last non-whitespace to trim trailing whitespace
        if !ch.is_ascii_whitespace() {
            end = next;
        }
    }

    if end == 0 {
        return PyProjectDependencySpecRanges {
            name: None,
            extras: None,
            version: None,
        };
    }

    // Name ends at whichever comes first: extras or version specifier
    let name_boundary = [extras_start, version_start]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(end);

    // Trim trailing whitespace from name by finding last non-ws byte
    let name_end = text.as_bytes()[..name_boundary]
        .iter()
        .rposition(|b| !b.is_ascii_whitespace())
        .map(|i| i + 1);

    let name = name_end.map(|end| base.sub(text, p(0), p(end)));
    let version = version_start.map(|start| base.sub(text, p(start), p(end)));
    let extras = match (extras_start, extras_end) {
        (Some(start), Some(end)) => Some(base.sub(text, p(start), p(end))),
        _ => None,
    };

    PyProjectDependencySpecRanges {
        name,
        version,
        extras,
    }
}

const fn p(col: usize) -> TsPoint {
    TsPoint {
        row: 0,
        column: col,
    }
}

use std::{cmp::Ordering, str::FromStr};

use tower_lsp::lsp_types::{Position, Range};

use super::query_utils::{range_contains, range_from_node};

/**
    A node in the tree-sitter parse tree.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    pub contents: T,
    pub range: Range,
}

impl<T> Node<T> {
    pub fn new(node: &tree_sitter::Node<'_>, contents: T) -> Self {
        let range = range_from_node(node);
        Self { contents, range }
    }

    pub fn new_raw(range: Range, contents: T) -> Self {
        Self { contents, range }
    }

    pub fn contains(&self, pos: Position) -> bool {
        range_contains(self.range, pos)
    }
}

impl Node<String> {
    pub fn string(node: &tree_sitter::Node<'_>, contents: impl Into<String>) -> Self {
        Self::new(node, contents.into())
    }
}

impl<S> Node<S>
where
    S: AsRef<str>,
{
    pub fn quoted(&self) -> &str {
        let s: &str = self.contents.as_ref();
        s
    }

    pub fn unquoted(&self) -> &str {
        let s = self.quoted();
        if let Some(s) = s.strip_prefix('"') {
            if let Some(s) = s.strip_suffix('"') {
                return s;
            }
        }
        s
    }

    pub fn parse<T: FromStr>(&self) -> Result<T, <T as FromStr>::Err> {
        self.unquoted().parse()
    }
}

/**
    The kind of dependency.
*/
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DependencyKind {
    #[default]
    Default,
    Dev,
    Build,
    Peer,
    Optional,
    Server,
}

/**
    The source of a dependency.
*/
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum DependencySource {
    #[default]
    Registry,
    Path {
        path: Node<String>,
    },
    Git {
        url: Node<String>,
    },
}

impl DependencySource {
    pub fn contents(&self) -> Option<&str> {
        match self {
            Self::Registry => None,
            Self::Path { path } => Some(path.contents.as_ref()),
            Self::Git { url } => Some(url.contents.as_ref()),
        }
    }
}

/**
    A dependency specification, containing:

    - The source of the dependency
    - The version of the dependency (may be `None` if the dependency is not versioned)
    - The features of the dependency (may also be `None` if the dependency has no features specified)
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencySpec {
    pub source: DependencySource,
    pub version: Option<Node<String>>,
    pub features: Option<Node<Vec<Node<String>>>>,
}

/**
    A partial *or* fully parsed dependency.

    Contains the kind of dependency, the name of the dependency,
    and the full version specification of the dependency.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dependency {
    Partial {
        kind: DependencyKind,
        name: Node<String>,
    },
    Full {
        kind: DependencyKind,
        name: Node<String>,
        spec: Node<DependencySpec>,
    },
}

impl Dependency {
    pub fn new_partial(kind: DependencyKind, name: Node<String>) -> Self {
        Self::Partial { kind, name }
    }

    pub fn new_full(kind: DependencyKind, name: Node<String>, spec: Node<DependencySpec>) -> Self {
        Self::Full { kind, name, spec }
    }

    pub fn new_opt(
        kind: DependencyKind,
        name: Node<String>,
        spec: Option<Node<DependencySpec>>,
    ) -> Self {
        match spec {
            Some(spec) => Self::new_full(kind, name, spec),
            None => Self::new_partial(kind, name),
        }
    }

    pub fn kind(&self) -> DependencyKind {
        match self {
            Self::Partial { kind, .. } => *kind,
            Self::Full { kind, .. } => *kind,
        }
    }

    pub fn name(&self) -> &Node<String> {
        match self {
            Self::Partial { name, .. } => name,
            Self::Full { name, .. } => name,
        }
    }

    pub fn spec(&self) -> Option<&Node<DependencySpec>> {
        match self {
            Self::Partial { .. } => None,
            Self::Full { spec, .. } => Some(spec),
        }
    }

    pub fn sort_vec(vec: &mut [Self]) {
        vec.sort_by(|a, b| match (a.spec(), b.spec()) {
            (Some(a), Some(b)) => {
                let a_range = a.range;
                let b_range = b.range;
                a_range
                    .start
                    .cmp(&b_range.start)
                    .then_with(|| a_range.end.cmp(&b_range.end))
            }
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        });
    }

    pub fn find_at_pos(vec: &[Self], pos: Position) -> Option<&Self> {
        vec.iter()
            .find(|dep| dep.name().contains(pos) || dep.spec().is_some_and(|s| s.contains(pos)))
    }
}

/**
    A fully parsed tool, containing:

    - The name of the tool
    - The spec of the tool
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tool {
    pub name: Node<String>,
    pub spec: Node<String>,
}

impl Tool {
    pub fn sort_vec(vec: &mut [Self]) {
        vec.sort_by(|a, b| {
            let a_range = a.name.range;
            let b_range = b.name.range;
            a_range
                .start
                .cmp(&b_range.start)
                .then_with(|| a_range.end.cmp(&b_range.end))
        });
    }

    pub fn find_at_pos(vec: &[Self], pos: Position) -> Option<&Self> {
        vec.iter()
            .find(|dep| dep.name.contains(pos) || dep.spec.contains(pos))
    }
}

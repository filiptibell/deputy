use std::str::FromStr;

use tower_lsp::lsp_types::Range;

use super::query_utils::range_from_node;

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
    A fully parsed dependency.

    Contains the kind of dependency, the name of the dependency,
    and the full version specification of the dependency.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub kind: DependencyKind,
    pub name: Node<String>,
    pub spec: Node<DependencySpec>,
}

impl Dependency {
    pub fn sort_vec(vec: &mut [Self]) {
        vec.sort_by(|a, b| {
            let a_range = a.spec.range;
            let b_range = b.spec.range;
            a_range
                .start
                .cmp(&b_range.start)
                .then_with(|| a_range.end.cmp(&b_range.end))
                .then_with(|| a.name.range.start.cmp(&b.name.range.start))
                .then_with(|| a.name.range.end.cmp(&b.name.range.end))
        });
    }
}

/**
    A fully parsed tool specification, containing:

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
}

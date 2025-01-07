use std::{cmp::Ordering, str::FromStr};

use tower_lsp::lsp_types::{Position, Range};

use crate::util::Versioned;

use super::query_utils::{range_contains, range_extend, range_for_substring, range_from_node};

/**
    A node in the tree-sitter parse tree.
*/
#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DependencySpec {
    pub source: DependencySource,
    pub version: Option<Node<String>>,
    pub features: Option<Node<Vec<Node<String>>>>,
}

impl Versioned for DependencySpec {
    fn raw_version_string(&self) -> String {
        self.version
            .clone()
            .unwrap_or_default()
            .unquoted()
            .to_string()
    }
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

impl Versioned for Dependency {
    fn raw_version_string(&self) -> String {
        self.spec()
            .cloned()
            .unwrap_or_default()
            .contents
            .raw_version_string()
    }
}

/**
    A fully parsed simple dependency, containing:

    - The name of the tool
    - The spec of the tool
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleDependency {
    pub kind: DependencyKind,
    pub name: Node<String>,
    pub spec: Node<String>,
}

impl SimpleDependency {
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

    pub fn parsed_spec(&self) -> ParsedSpec {
        ParsedSpec::from(self.spec.clone())
    }
}

impl Versioned for SimpleDependency {
    fn raw_version_string(&self) -> String {
        self.parsed_spec()
            .version
            .unwrap_or_default()
            .unquoted()
            .to_string()
    }
}

/**
    A parsed tool specification, in the format:

    ```
    "author/name@version"
    ```

    Note that this is not guaranteed to be fully parsed, only partial.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSpec {
    pub author: Node<String>,
    pub name: Option<Node<String>>,
    pub version: Option<Node<String>>,
}

impl ParsedSpec {
    pub fn into_full(self) -> Option<ParsedSpecFull> {
        let name = self.name?;
        let version = self.version?;
        Some(ParsedSpecFull {
            author: self.author,
            name,
            version,
        })
    }
}

impl Versioned for ParsedSpec {
    fn raw_version_string(&self) -> String {
        self.version
            .clone()
            .unwrap_or_default()
            .unquoted()
            .to_string()
    }
}

impl From<Node<String>> for ParsedSpec {
    fn from(node: Node<String>) -> ParsedSpec {
        let raw = node.unquoted();

        let (owner, repository, version) = if let Some((owner, rest)) = raw.split_once('/') {
            if let Some((repository, version)) = rest.split_once('@') {
                (owner, Some(repository), Some(version))
            } else {
                (owner, Some(rest), None)
            }
        } else {
            (raw, None, None)
        };

        ParsedSpec {
            author: Node::new_raw(
                range_for_substring(node.range, node.quoted(), owner),
                owner.to_string(),
            ),
            name: repository.map(|repository| {
                Node::new_raw(
                    range_for_substring(node.range, node.quoted(), repository),
                    repository.to_string(),
                )
            }),
            version: version.map(|version| {
                Node::new_raw(
                    range_for_substring(node.range, node.quoted(), version),
                    version.to_string(),
                )
            }),
        }
    }
}

/**
    A *fully* parsed tool specification, in the format:

    ```
    "author/name@version"
    ```

    Contains all fully parsed fields, unlike `ToolSpecParsed`.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSpecFull {
    pub author: Node<String>,
    pub name: Node<String>,
    pub version: Node<String>,
}

impl ParsedSpecFull {
    pub fn range(&self) -> Range {
        range_extend(self.author.range, self.version.range)
    }
}

impl Versioned for ParsedSpecFull {
    fn raw_version_string(&self) -> String {
        self.version.unquoted().to_string()
    }
}

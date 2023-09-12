#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GitTreeRoot {
    pub sha: String,
    pub url: String,
    pub tree: Vec<GitTreeNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitTreeNode {
    pub sha: String,
    pub url: String,
    #[serde(rename = "type")]
    pub kind: GitNodeKind,
    pub size: u64,
    pub path: String,
}

impl GitTreeNode {
    pub const fn is_blob(&self) -> bool {
        matches!(self.kind, GitNodeKind::Blob)
    }

    pub const fn is_tree(&self) -> bool {
        matches!(self.kind, GitNodeKind::Tree)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitNodeKind {
    Blob,
    Tree,
}

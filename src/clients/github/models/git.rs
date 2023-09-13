#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GitTreeRoot {
    pub sha: String,
    pub url: String,
    pub tree: Vec<GitTreeNode>,
}

impl GitTreeRoot {
    pub fn find_node_by_path(&self, path: &str) -> Option<GitTreeNode> {
        self.tree.iter().find_map(|node| {
            if node.path.to_ascii_lowercase().eq_ignore_ascii_case(path) {
                Some(node.clone())
            } else {
                None
            }
        })
    }

    pub fn get_directory_paths(&self) -> Vec<String> {
        self.tree
            .iter()
            .filter_map(|node| {
                if node.is_tree() {
                    Some(node.path.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_file_paths_excluding_json(&self) -> Vec<String> {
        self.tree
            .iter()
            .filter_map(|node| {
                if node.is_blob() && !node.path.ends_with(".json") {
                    Some(node.path.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitTreeNode {
    pub sha: String,
    pub url: String,
    #[serde(rename = "type")]
    pub kind: GitNodeKind,
    pub size: Option<u64>,
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

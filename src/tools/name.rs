use std::str::FromStr;

use tower_lsp::lsp_types::*;

use crate::util::LspUriExt;

#[derive(Debug, Clone, Copy)]
pub enum ToolName {
    Aftman,
    Cargo,
    Foreman,
    Wally,
}

impl ToolName {
    pub fn from_uri(uri: &Url) -> Result<Self, &'static str> {
        match uri.file_name() {
            Some(file_name) => file_name.parse(),
            None => Err("No file name"),
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Aftman, Self::Cargo, Self::Foreman, Self::Wally]
    }

    pub fn file_glob(&self) -> &'static str {
        match self {
            Self::Aftman => "**/aftman.toml",
            Self::Cargo => "**/Cargo.{toml,lock}",
            Self::Foreman => "**/foreman.toml",
            Self::Wally => "**/wally.toml",
        }
    }

    pub fn relevant_file_uris(&self, uri: &Url) -> Vec<Url> {
        match self {
            Self::Aftman => Vec::new(),
            Self::Cargo => vec![match uri.file_name().as_deref() {
                Some("Cargo.toml") => uri.with_file_name("Cargo.lock").unwrap(),
                Some("cargo.toml") => uri.with_file_name("cargo.lock").unwrap(),
                Some("Cargo.lock") => uri.with_file_name("Cargo.toml").unwrap(),
                Some("cargo.lock") => uri.with_file_name("cargo.toml").unwrap(),
                _ => return Vec::new(),
            }],
            Self::Foreman => Vec::new(),
            Self::Wally => Vec::new(),
        }
    }
}

impl FromStr for ToolName {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_ref() {
            "aftman" | "aftman.toml" => Ok(Self::Aftman),
            "cargo" | "cargo.toml" | "cargo.lock" => Ok(Self::Cargo),
            "foreman" | "foreman.toml" => Ok(Self::Foreman),
            "wally" | "wally.toml" => Ok(Self::Wally),
            _ => Err("Unknown tool"),
        }
    }
}

use std::str::FromStr;

use tower_lsp::lsp_types::*;

use crate::util::uri_to_file_name;

#[derive(Debug, Clone, Copy)]
pub enum ToolName {
    Aftman,
    Cargo,
    Foreman,
    Wally,
}

impl ToolName {
    pub fn from_uri(uri: &Url) -> Result<Self, &'static str> {
        match uri_to_file_name(uri) {
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
            Self::Cargo => "**/Cargo.toml",
            Self::Foreman => "**/foreman.toml",
            Self::Wally => "**/wally.toml",
        }
    }
}

impl FromStr for ToolName {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_ref() {
            "aftman" | "aftman.toml" => Ok(Self::Aftman),
            "cargo" | "Cargo" | "Cargo.toml" => Ok(Self::Cargo),
            "foreman" | "foreman.toml" => Ok(Self::Foreman),
            "wally" | "wally.toml" => Ok(Self::Wally),
            _ => Err("Unknown tool"),
        }
    }
}

use std::str::FromStr;

use tower_lsp::lsp_types::*;

use crate::util::LspUriExt;

use super::NPM_LOCKFILE_FILE_NAMES;

#[derive(Debug, Clone, Copy)]
pub enum ToolName {
    Aftman,
    Cargo,
    Foreman,
    Npm,
    Rokit,
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
        vec![
            Self::Aftman,
            Self::Cargo,
            Self::Foreman,
            Self::Npm,
            Self::Rokit,
            Self::Wally,
        ]
    }

    pub fn file_glob(&self) -> &'static str {
        match self {
            Self::Aftman => "**/aftman.toml",
            Self::Cargo => "**/Cargo.{toml,lock}",
            Self::Foreman => "**/foreman.toml",
            Self::Npm => "**/package.json",
            Self::Rokit => "**/rokit.toml",
            Self::Wally => "**/wally.toml",
        }
    }

    pub fn relevant_file_uris(&self, uri: &Url) -> Vec<Url> {
        match self {
            Self::Aftman => Vec::new(),
            Self::Cargo => match uri.file_name().as_deref() {
                Some("Cargo.lock") => vec![uri.with_file_name("Cargo.toml").unwrap()],
                Some("Cargo.toml") => {
                    let mut lockfiles = Vec::new();
                    let mut current_dir = uri.to_file_path().unwrap();
                    loop {
                        current_dir.pop();
                        let lockfile = current_dir.join("Cargo.lock");
                        if lockfile.exists() {
                            lockfiles.push(Url::from_file_path(lockfile).unwrap());
                        }
                        if !current_dir.pop() {
                            break;
                        }
                    }
                    lockfiles
                }
                _ => Vec::new(),
            },
            Self::Foreman => Vec::new(),
            Self::Npm => match uri.file_name().as_deref() {
                Some("package.json") => NPM_LOCKFILE_FILE_NAMES
                    .iter()
                    .map(|name| uri.with_file_name(name).unwrap())
                    .collect(),
                Some(f) if NPM_LOCKFILE_FILE_NAMES.contains(&f) => {
                    vec![uri.with_file_name("package.json").unwrap()]
                }
                _ => Vec::new(),
            },
            Self::Rokit => Vec::new(),
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
            "package.json" | "package-lock.json" => Ok(Self::Npm),
            "rokit" | "rokit.toml" => Ok(Self::Rokit),
            "wally" | "wally.toml" => Ok(Self::Wally),
            _ => Err("Unknown tool"),
        }
    }
}

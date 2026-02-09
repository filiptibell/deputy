use std::path::Path;

use async_language_server::lsp_types::Url;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

use deputy_parser::utils::unquote;
use deputy_versioning::{Version, VersionReq, Versioned};

use super::Clients;

pub async fn get_features(clients: &Clients, dname: &str, dver: &str) -> Option<Vec<String>> {
    let dreq = VersionReq::parse(dver).ok()?;

    let metas = clients
        .crates
        .get_sparse_index_crate_metadatas(dname)
        .await
        .inspect_err(|e| {
            tracing::error!("failed to get crate data for {dname}: {e}");
        })
        .ok()?;

    let meta = metas.iter().find_map(|meta| {
        let version = meta.parse_version().ok()?;
        if dreq.matches(&version) {
            Some(meta)
        } else {
            None
        }
    })?;

    Some(
        meta.all_features()
            .into_iter()
            .map(ToString::to_string)
            .collect(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalCrateParseState {
    Other,
    Package,
    Features,
}

#[derive(Debug, Clone)]
pub struct LocalCrate {
    pub version: Option<Version>,
    pub features: Vec<String>,
}

impl LocalCrate {
    pub async fn read(doc_url: &Url, relative_path: &str) -> Option<Self> {
        let doc_path = doc_url.to_file_path().ok()?;
        let doc_dir = doc_path.parent()?;

        let dep_path = doc_dir.join(relative_path);
        let manifest_path = if dep_path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("toml"))
        {
            dep_path
        } else {
            dep_path.join("Cargo.toml")
        };

        Self::parse_local_manifest(&manifest_path).await
    }

    async fn parse_local_manifest(path: &Path) -> Option<Self> {
        let file = File::open(path).await.ok()?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut version = None;
        let mut features = Vec::new();
        let mut section = LocalCrateParseState::Other;

        while let Ok(Some(line)) = lines.next_line().await {
            let trimmed = line.trim();

            if trimmed.starts_with('[') {
                section = match trimmed {
                    "[package]" => LocalCrateParseState::Package,
                    "[features]" => LocalCrateParseState::Features,
                    _ => LocalCrateParseState::Other,
                };
                continue;
            }

            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };
            let key = key.trim();

            match section {
                LocalCrateParseState::Package if key == "version" => {
                    version = unquote(value.trim()).parse().ok();
                }
                LocalCrateParseState::Features => {
                    features.push(key.to_string());
                }
                _ => {}
            }
        }

        Some(Self { version, features })
    }
}

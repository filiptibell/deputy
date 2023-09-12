use std::collections::{HashSet, VecDeque};

use tower_lsp::lsp_types::Url;
use tracing::debug;

use super::models::*;
use super::*;

impl WallyClient {
    pub async fn get_index_config(&self, index_url: &str) -> RequestResult<IndexConfig> {
        let (owner, repo) = parse_index_url(index_url)?;

        let bytes = self
            .github
            .get_repository_file(&owner, &repo, "config.json")
            .await?;

        Ok(serde_json::from_slice::<IndexConfig>(&bytes)?)
    }

    pub async fn get_index_configs_following_fallbacks(
        &self,
        index_url: &str,
    ) -> RequestResult<Vec<(String, IndexConfig)>> {
        let mut pending = VecDeque::new();
        pending.push_back(index_url.to_ascii_lowercase());

        let mut visited = HashSet::new();
        let mut results = Vec::new();
        while let Some(pending_url) = pending.pop_front() {
            let pending_conf = self.get_index_config(&pending_url).await?;
            for fallback_url in &pending_conf.fallback_registries {
                let fallback_low = fallback_url.to_ascii_lowercase();
                if !visited.contains(&fallback_low) {
                    visited.insert(fallback_low.clone());
                    pending.push_back(fallback_low);
                }
            }
            visited.insert(pending_url.clone());
            results.push((pending_url, pending_conf));
        }

        Ok(results)
    }

    pub async fn get_index_scopes(&self, index_url: &str) -> RequestResult<Vec<String>> {
        let (owner, repo) = parse_index_url(index_url)?;

        let root = self
            .github
            .get_repository_tree(&owner, &repo, "main") // FUTURE: Fetch proper default branch
            .await?;

        Ok(root
            .tree
            .into_iter()
            .filter_map(|node| {
                if node.is_tree() {
                    Some(node.path)
                } else {
                    None
                }
            })
            .collect())
    }

    pub async fn get_index_packages(
        &self,
        index_url: &str,
        scope: &str,
    ) -> RequestResult<Vec<String>> {
        let (owner, repo) = parse_index_url(index_url)?;
        let scope_low = scope.to_ascii_lowercase();

        // Fetch the root of the index so that we can search through
        // nodes and find the sha hash of an inner git tree to fetch
        let root = self
            .github
            .get_repository_tree(&owner, &repo, "main") // FUTURE: Fetch proper default branch
            .await?;

        let scope_sha = root
            .tree
            .iter()
            .find_map(|node| {
                if node.path.eq_ignore_ascii_case(&scope_low) {
                    Some(node.sha.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                RequestError::Custom(format!("No packages were found for scope '{scope_low}'"))
            })?;

        // If we find a git node for the given package
        // scope, we try to then fetch that full subtree
        let root_for_scope = self
            .github
            .get_repository_tree(&owner, &repo, &scope_sha)
            .await?;

        Ok(root_for_scope
            .tree
            .into_iter()
            .filter_map(|node| {
                if node.is_blob() && !node.path.ends_with(".json") {
                    Some(node.path)
                } else {
                    None
                }
            })
            .collect())
    }

    pub async fn get_index_metadatas(
        &self,
        index_url: &str,
        scope: &str,
        name: &str,
    ) -> RequestResult<Vec<Metadata>> {
        let (owner, repo) = parse_index_url(index_url)?;
        let scope_low = scope.to_ascii_lowercase();
        let name_low = name.to_ascii_lowercase();

        let bytes = self
            .github
            .get_repository_file(&owner, &repo, &format!("{scope_low}/{name_low}"))
            .await?;

        let text = String::from_utf8(bytes.to_vec())?;
        let mut metas = Metadata::try_from_lines(text.lines().collect())?;

        // NOTE: We should sort by most recent version first
        metas.reverse();

        Ok(metas)
    }

    // TODO: Add get_package_metadatas as an alternative to
    // get_index_metadatas, using the wally rest api instead,
    // this will let us use less of the github request budget
}

fn parse_index_url(index_url: &str) -> RequestResult<(String, String)> {
    let url = index_url.to_ascii_lowercase();
    let url = Url::parse(&url)?;

    if let Some(stripped) = url
        .to_string()
        .trim_end_matches(".git")
        .strip_prefix("https://github.com/")
    {
        if let Some((owner, repo)) = stripped.split_once('/') {
            return Ok((owner.to_string(), repo.to_string()));
        }
    }

    Err(RequestError::Client(format!(
        "malformed index url - failed to parse github owner & repo from '{url}'"
    )))
}

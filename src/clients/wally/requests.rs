use std::collections::{HashSet, VecDeque};

use async_language_server::lsp_types::Url;
use reqwest::StatusCode;
use tracing::info;

use super::models::{IndexConfig, Metadata};
use super::{RequestError, RequestResult, ResponseError, WallyClient};

const DEFAULT_INDEX_BRANCH: &str = "main";

impl WallyClient {
    async fn get_index_config(&self, index_url: &str) -> RequestResult<IndexConfig> {
        let (owner, repo) = parse_index_url(index_url)?;

        let fut = async {
            let bytes = self
                .github
                .get_repository_file(&owner, &repo, "config.json")
                .await?;
            let config = serde_json::from_slice::<IndexConfig>(&bytes)?;

            info!("Wally registry config found: {config:#?}");

            Ok(config)
        };

        self.cache
            .index_configs
            .with_caching(format!("{owner}/{repo}"), fut)
            .await
    }

    async fn get_index_configs_following_fallbacks(
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

    async fn get_index_urls_following_fallbacks(
        &self,
        index_url: &str,
    ) -> RequestResult<Vec<String>> {
        let index_configs = self
            .get_index_configs_following_fallbacks(index_url)
            .await?;

        Ok(index_configs
            .into_iter()
            .map(|(k, _)| k)
            .collect::<Vec<_>>())
    }

    pub async fn get_index_scopes(&self, index_url: &str) -> RequestResult<Vec<String>> {
        let mut all_scopes = HashSet::new();

        for index_url in self.get_index_urls_following_fallbacks(index_url).await? {
            let (owner, repo) = parse_index_url(&index_url)?;

            let root = self
                .github
                .get_repository_tree(&owner, &repo, DEFAULT_INDEX_BRANCH)
                .await?;

            all_scopes.extend(root.get_directory_paths());
        }

        Ok(Vec::from_iter(all_scopes))
    }

    pub async fn get_index_packages(
        &self,
        index_url: &str,
        scope: &str,
    ) -> RequestResult<Vec<String>> {
        let scope_low = scope.to_ascii_lowercase();

        let mut scope_exists = false;
        let mut scope_paths = Vec::new();

        for index_url in self.get_index_urls_following_fallbacks(index_url).await? {
            let (owner, repo) = parse_index_url(&index_url)?;

            let res = self
                .github
                .get_repository_tree(&owner, &repo, DEFAULT_INDEX_BRANCH)
                .await;

            if let Ok(Some(scope_node)) = res.map(|root| root.find_node_by_path(&scope_low)) {
                let root_for_scope = self
                    .github
                    .get_repository_tree(&owner, &repo, &scope_node.sha)
                    .await?;

                scope_exists = true;
                scope_paths.extend(root_for_scope.get_file_paths_excluding_json());
            }
        }

        if scope_exists {
            Ok(scope_paths)
        } else {
            Err(RequestError::Response(
                ResponseError::from_status_and_string(
                    StatusCode::NOT_FOUND,
                    format!("No packages were found for scope `{scope_low}`"),
                ),
            ))
        }
    }

    pub async fn get_index_metadatas(
        &self,
        index_url: &str,
        scope: &str,
        name: &str,
    ) -> RequestResult<Vec<Metadata>> {
        let scope_low = scope.to_ascii_lowercase();
        let name_low = name.to_ascii_lowercase();

        for index_url in self.get_index_urls_following_fallbacks(index_url).await? {
            let (owner, repo) = parse_index_url(&index_url)?;

            let res = self
                .github
                .get_repository_file(&owner, &repo, &format!("{scope_low}/{name_low}"))
                .await;

            match res {
                Err(_) => {}
                Ok(bytes) => {
                    let text = String::from_utf8(bytes.clone())?;
                    let mut metas = Metadata::try_from_lines(text.lines().collect())?;

                    metas.reverse(); // NOTE: We should sort by most recent version first

                    return Ok(metas);
                }
            }
        }

        Err(RequestError::Response(
            ResponseError::from_status_and_string(
                StatusCode::NOT_FOUND,
                format!("No metadatas were found for package `{scope_low}`{scope_low}'"),
            ),
        ))
    }
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
            return Ok((owner.to_ascii_lowercase(), repo.to_ascii_lowercase()));
        }
    }

    Err(RequestError::Client(format!(
        "malformed index url - failed to parse github owner & repo from `{url}`"
    )))
}

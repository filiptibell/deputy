use tracing::error;

use octocrab::{
    models::{repos::Release, Repository, RepositoryMetrics},
    params::repos::Type as RepoType,
};

use super::*;

impl GithubWrapper {
    pub async fn list_repositories_for(
        &self,
        owner: impl Into<String>,
    ) -> GithubResult<Vec<Repository>> {
        let owner = owner.into();

        let client = self.client.clone();
        let cache = self.cache.clone();

        let cache_map = &cache.repository_listings;
        let cache_key = owner.trim().to_ascii_lowercase();

        if let Some(cached) = cache_map.get(&cache_key) {
            return cached.clone();
        }

        let client = client.lock().await;
        let result = client
            .orgs(owner)
            .list_repos()
            .repo_type(RepoType::All)
            .per_page(50)
            .send()
            .await
            .map_err(GithubError::from);

        if let Err(e) = &result {
            if e.is_rate_limit_error() {
                self.notify_rate_limited();
            } else {
                error!("GitHub error: {e}");
            }
        }

        let result = result.map(|list| list.items);
        cache_map.insert(cache_key, result.clone()).await;
        result
    }

    pub async fn get_repository_metrics(
        &self,
        owner: impl Into<String>,
        repository: impl Into<String>,
    ) -> GithubResult<RepositoryMetrics> {
        let owner = owner.into();
        let repository = repository.into();

        let client = self.client.clone();
        let cache = self.cache.clone();

        let cache_map = &cache.repository_metrics;
        let cache_key = format!(
            "{}/{}",
            owner.trim().to_ascii_lowercase(),
            repository.trim().to_ascii_lowercase()
        );

        if let Some(cached) = cache_map.get(&cache_key) {
            return cached.clone();
        }

        let client = client.lock().await;
        let result = client
            .repos(owner, repository)
            .get_community_profile_metrics()
            .await
            .map_err(GithubError::from);

        if let Err(e) = &result {
            if e.is_rate_limit_error() {
                self.notify_rate_limited();
            } else {
                error!("GitHub error: {e}");
            }
        }

        cache_map.insert(cache_key, result.clone()).await;
        result
    }

    pub async fn get_repository_releases(
        &self,
        owner: impl Into<String>,
        repository: impl Into<String>,
    ) -> GithubResult<Vec<Release>> {
        let owner = owner.into();
        let repository = repository.into();

        let client = self.client.clone();
        let cache = self.cache.clone();

        let cache_map = &cache.repository_releases;
        let cache_key = format!(
            "{}/{}",
            owner.trim().to_ascii_lowercase(),
            repository.trim().to_ascii_lowercase()
        );

        if let Some(cached) = cache_map.get(&cache_key) {
            return cached.clone();
        }

        let client = client.lock().await;
        let result = client
            .repos(owner, repository)
            .releases()
            .list()
            .send()
            .await
            .map_err(GithubError::from);

        if let Err(e) = &result {
            if e.is_rate_limit_error() {
                self.notify_rate_limited();
            } else {
                error!("GitHub error: {e}");
            }
        }

        let result = result.map(|list| list.items);
        cache_map.insert(cache_key, result.clone()).await;
        result
    }
}

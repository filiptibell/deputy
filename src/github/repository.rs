use octocrab::models::RepositoryMetrics;

use super::*;

impl GithubWrapper {
    pub async fn get_repository_metrics(
        &self,
        owner: impl Into<String>,
        repository: impl Into<String>,
    ) -> GithubResult<RepositoryMetrics> {
        let owner = owner.into();
        let repository = repository.into();

        let cache = self.cache.clone();
        let cache_map = &cache.repository_metrics;
        let cache_key = format!("{}/{}", &owner, &repository);

        if let Some(cached) = cache.get(cache_map, &cache_key).await {
            return cached.clone();
        }

        let result = self
            .client
            .repos(owner, repository)
            .get_community_profile_metrics()
            .await
            .map_err(GithubError::from);

        cache.add(cache_map, cache_key, result, false).await
    }
}

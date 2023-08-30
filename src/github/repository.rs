use octocrab::models::{repos::Release, RepositoryMetrics};
use tracing::error;

use super::*;

impl GithubWrapper {
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
        let cache_key = format!("{}/{}", &owner, &repository);

        if let Some(cached) = cache.get(cache_map, &cache_key).await {
            return cached.clone();
        }

        let client = client.lock().await;
        let result = client
            .repos(owner, repository)
            .get_community_profile_metrics()
            .await;

        if let Err(e) = &result {
            if is_rate_limit_error(e) {
                self.notify_rate_limited();
            } else {
                error!("GitHub error: {e}");
            }
        }

        cache
            .add(
                cache_map,
                cache_key,
                result.map_err(GithubError::from),
                false,
            )
            .await
    }

    pub async fn get_latest_release(
        &self,
        owner: impl Into<String>,
        repository: impl Into<String>,
    ) -> GithubResult<Release> {
        let owner = owner.into();
        let repository = repository.into();

        let client = self.client.clone();
        let cache = self.cache.clone();

        let cache_map = &cache.latest_releases;
        let cache_key = format!("{}/{}", &owner, &repository);

        if let Some(cached) = cache.get(cache_map, &cache_key).await {
            return cached.clone();
        }

        let client = client.lock().await;
        let result = client
            .repos(owner, repository)
            .releases()
            .get_latest()
            .await;

        if let Err(e) = &result {
            if is_rate_limit_error(e) {
                self.notify_rate_limited();
            } else {
                error!("GitHub error: {e}");
            }
        }

        cache
            .add(
                cache_map,
                cache_key,
                result.map_err(GithubError::from),
                true,
            )
            .await
    }
}

fn is_rate_limit_error(error: &octocrab::Error) -> bool {
    if let octocrab::Error::GitHub { source, .. } = error {
        let message = source.message.to_ascii_lowercase();
        message.contains("rate limit exceeded")
            || message.contains("higher rate limit")
            || message.contains("#rate-limiting")
    } else {
        false
    }
}

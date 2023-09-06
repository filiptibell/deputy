use reqwest::Method;
use tracing::debug;

use super::consts::*;
use super::models::*;
use super::*;

impl GithubClient {
    pub async fn get_repository_metrics(
        &self,
        owner: &str,
        repository: &str,
    ) -> RequestResult<RepositoryMetrics> {
        let owner_low = owner.to_ascii_lowercase();
        let repository_low = repository.to_ascii_lowercase();

        let metrics_url =
            format!("{GITHUB_API_BASE_URL}/repos/{owner_low}/{repository_low}/community/profile");

        let fut = async move {
            debug!("Fetching GitHub metrics for {owner}/{repository}");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let (status, bytes) = self.request(Method::GET, &metrics_url).await?;
                if !status.is_success() {
                    return Err(ResponseError::from((status, bytes)).into());
                }
                Ok(serde_json::from_slice::<RepositoryMetrics>(&bytes)?)
            }
            .await;

            self.emit_result(&inner);

            inner
        };

        self.cache
            .repository_metrics
            .with_caching(format!("{owner}/{repository}"), fut)
            .await
    }

    pub async fn get_repository_releases(
        &self,
        owner: &str,
        repository: &str,
    ) -> RequestResult<Vec<RepositoryRelease>> {
        let owner_low = owner.to_ascii_lowercase();
        let repository_low = repository.to_ascii_lowercase();

        let releases_url =
            format!("{GITHUB_API_BASE_URL}/repos/{owner_low}/{repository_low}/releases");

        let fut = async move {
            debug!("Fetching GitHub releases for {owner}/{repository}");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let (status, bytes) = self.request(Method::GET, &releases_url).await?;
                if !status.is_success() {
                    return Err(ResponseError::from((status, bytes)).into());
                }
                Ok(serde_json::from_slice::<Vec<RepositoryRelease>>(&bytes)?)
            }
            .await;

            self.emit_result(&inner);

            inner
        };

        self.cache
            .repository_releases
            .with_caching(format!("{owner}/{repository}"), fut)
            .await
    }
}

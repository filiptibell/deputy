use tracing::debug;

use super::models::*;
use super::*;

pub const GITHUB_API_BASE_URL: &str = "https://api.github.com";
pub const GITHUB_API_USER_AGENT: &str =
    concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION"),);
pub const GITHUB_API_CONTENT_TYPE: &str = "application/vnd.github.v3+json";

impl GithubWrapper {
    pub async fn get_repository_metrics(
        &self,
        owner: &str,
        repository: &str,
    ) -> GithubResult<RepositoryMetrics> {
        let owner_low = owner.to_ascii_lowercase();
        let repository_low = repository.to_ascii_lowercase();

        let metrics_url =
            format!("{GITHUB_API_BASE_URL}/repos/{owner_low}/{repository_low}/community/profile");

        let fut = async move {
            debug!("Fetching GitHub metrics for {owner}/{repository}");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let response = self.client().get(&metrics_url).send().await?;
                let status = response.status();
                let bytes = response.bytes().await?;
                if !status.is_success() {
                    return Err(GithubError::new(format!(
                        "{} {} - {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap(),
                        String::from_utf8_lossy(&bytes)
                    )));
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
    ) -> GithubResult<Vec<RepositoryRelease>> {
        let owner_low = owner.to_ascii_lowercase();
        let repository_low = repository.to_ascii_lowercase();

        let releases_url =
            format!("{GITHUB_API_BASE_URL}/repos/{owner_low}/{repository_low}/releases");

        let fut = async move {
            debug!("Fetching GitHub releases for {owner}/{repository}");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let response = self.client().get(&releases_url).send().await?;
                let status = response.status();
                let bytes = response.bytes().await?;
                if !status.is_success() {
                    return Err(GithubError::new(format!(
                        "{} {} - {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap(),
                        String::from_utf8_lossy(&bytes)
                    )));
                }
                // TODO: Handle pages
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

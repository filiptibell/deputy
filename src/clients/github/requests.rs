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
                let bytes = self.request_get(&metrics_url).await?;
                Ok(serde_json::from_slice::<RepositoryMetrics>(&bytes)?)
            }
            .await;

            self.emit_result(&inner);

            inner
        };

        self.cache
            .repository_metrics
            .with_caching(format!("{owner_low}/{repository_low}"), fut)
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
                let bytes = self.request_get(&releases_url).await?;
                Ok(serde_json::from_slice::<Vec<RepositoryRelease>>(&bytes)?)
            }
            .await;

            self.emit_result(&inner);

            inner
        };

        self.cache
            .repository_releases
            .with_caching(format!("{owner_low}/{repository_low}"), fut)
            .await
    }

    pub async fn get_repository_tree(
        &self,
        owner: &str,
        repository: &str,
        sha: &str,
    ) -> RequestResult<GitTreeRoot> {
        let owner_low = owner.to_ascii_lowercase();
        let repository_low = repository.to_ascii_lowercase();
        let sha_low = sha.to_ascii_lowercase();

        let git_tree_url =
            format!("{GITHUB_API_BASE_URL}/repos/{owner_low}/{repository_low}/git/trees/{sha_low}");

        let fut = async move {
            debug!("Fetching GitHub tree for {owner}/{repository}/{sha}");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let bytes = self.request_get(&git_tree_url).await?;
                Ok(serde_json::from_slice::<GitTreeRoot>(&bytes)?)
            }
            .await;

            self.emit_result(&inner);

            inner
        };

        self.cache
            .repository_trees
            .with_caching(format!("{owner_low}/{repository_low}/{sha_low}"), fut)
            .await
    }

    pub async fn get_repository_file(
        &self,
        owner: &str,
        repository: &str,
        path: &str,
    ) -> RequestResult<Vec<u8>> {
        let owner_low = owner.to_ascii_lowercase();
        let repository_low = repository.to_ascii_lowercase();

        let git_file_url =
            format!("{GITHUB_API_BASE_URL}/repos/{owner_low}/{repository_low}/contents/{path}");

        let agent_auth = self.auth_token.lock().unwrap().clone();
        let fut = async move {
            debug!("Fetching GitHub file for {owner}/{repository} at {path}");

            let result = Request::get(git_file_url)
                .with_header("Accept", consts::GITHUB_API_CONTENT_TYPE_RAW)
                .with_header_opt("Authorization", agent_auth)
                .send()
                .await;

            self.emit_result(&result);

            result
        };

        self.cache
            .repository_files
            .with_caching(format!("{owner_low}/{repository_low}/{path}"), fut)
            .await
    }
}

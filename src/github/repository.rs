use octocrab::models::{repos::Release, RepositoryMetrics};

use super::*;

impl GithubWrapper {
    pub async fn get_repository_metrics(
        &self,
        owner: &str,
        repository: &str,
    ) -> GithubResult<RepositoryMetrics> {
        let fut = async move {
            let result = self
                .client()
                .repos(owner, repository)
                .get_community_profile_metrics()
                .await
                .map_err(GithubError::from);

            self.emit_result(&result);

            result
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
    ) -> GithubResult<Vec<Release>> {
        let fut = async move {
            let result = self
                .client()
                .repos(owner, repository)
                .releases()
                .list()
                .send()
                .await
                .map(|r| r.items)
                .map_err(GithubError::from);

            self.emit_result(&result);

            result
        };

        self.cache
            .repository_releases
            .with_caching(format!("{owner}/{repository}"), fut)
            .await
    }
}

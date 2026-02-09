use tracing::debug;

use crate::github::models::RepositoryMetrics;
use crate::shared::RequestError;

use super::consts::BASE_URL_PROXY;
use super::models::ModuleVersion;
use super::util::{encode_module_path, extract_github_owner_repo};
use super::{GolangClient, RequestResult};

impl GolangClient {
    #[allow(clippy::missing_errors_doc)]
    pub async fn get_module_versions(&self, module: &str) -> RequestResult<Vec<String>> {
        let encoded = encode_module_path(module);
        let url = format!("{BASE_URL_PROXY}/{encoded}/@v/list");

        let fut = async {
            debug!("Fetching Go module versions for '{module}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let bytes = self.request_get(&url).await?;
                let text = String::from_utf8(bytes)?;
                let versions: Vec<String> = text
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                Ok(versions)
            }
            .await;

            GolangClient::emit_result(&inner);

            inner
        };

        self.cache
            .version_lists
            .with_caching(encoded.clone(), fut)
            .await
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn get_module_latest(&self, module: &str) -> RequestResult<ModuleVersion> {
        let encoded = encode_module_path(module);
        let url = format!("{BASE_URL_PROXY}/{encoded}/@latest");

        let fut = async {
            debug!("Fetching Go module latest for '{module}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let bytes = self.request_get(&url).await?;
                Ok(serde_json::from_slice::<ModuleVersion>(&bytes)?)
            }
            .await;

            GolangClient::emit_result(&inner);

            inner
        };

        self.cache
            .latest_versions
            .with_caching(encoded.clone(), fut)
            .await
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn get_module_metadata(&self, module: &str) -> RequestResult<RepositoryMetrics> {
        let (owner, repo) = extract_github_owner_repo(module).ok_or_else(|| {
            RequestError::Client(format!(
                "Cannot fetch metadata for non-GitHub module: {module}"
            ))
        })?;

        self.github.get_repository_metrics(&owner, &repo).await
    }
}

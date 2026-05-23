use tracing::debug;
use url::form_urlencoded;

use super::consts::BASE_URL_PKGSITE_API;
use super::models::{Module, Package, SearchResults, Versions};
use super::{GolangClient, RequestResult};

impl GolangClient {
    #[allow(clippy::missing_errors_doc)]
    pub async fn search(&self, query: &str) -> RequestResult<SearchResults> {
        let query = query.trim();
        let encoded_query = form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
        let url = format!("{BASE_URL_PKGSITE_API}/search?q={encoded_query}&limit=64");

        let fut = async {
            debug!("Searching pkg.go.dev for '{query}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let bytes = self.request_get(&url).await?;
                Ok(serde_json::from_slice::<SearchResults>(&bytes)?)
            }
            .await;

            GolangClient::emit_result(&inner);

            inner
        };

        self.cache
            .searches
            .with_caching(encoded_query.clone(), fut)
            .await
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn get_module(&self, module: &str) -> RequestResult<Module> {
        let module = module.trim();
        let url = format!("{BASE_URL_PKGSITE_API}/module/{module}");

        let fut = async {
            debug!("Fetching pkg.go.dev module metadata for '{module}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let bytes = self.request_get(&url).await?;
                Ok(serde_json::from_slice::<Module>(&bytes)?)
            }
            .await;

            GolangClient::emit_result(&inner);

            inner
        };

        self.cache
            .modules
            .with_caching(module.to_string(), fut)
            .await
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn get_package(&self, package: &str) -> RequestResult<Package> {
        let package = package.trim();
        let url = format!("{BASE_URL_PKGSITE_API}/package/{package}");

        let fut = async {
            debug!("Fetching pkg.go.dev package metadata for '{package}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let bytes = self.request_get(&url).await?;
                Ok(serde_json::from_slice::<Package>(&bytes)?)
            }
            .await;

            GolangClient::emit_result(&inner);

            inner
        };

        self.cache
            .packages
            .with_caching(package.to_string(), fut)
            .await
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn get_module_versions(&self, module: &str) -> RequestResult<Versions> {
        let module = module.trim();
        let url = format!("{BASE_URL_PKGSITE_API}/versions/{module}");

        let fut = async {
            debug!("Fetching pkg.go.dev module versions for '{module}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let bytes = self.request_get(&url).await?;
                let mut versions = serde_json::from_slice::<Versions>(&bytes)?;
                versions
                    .items
                    .retain(|version| version.module_path.eq_ignore_ascii_case(module));
                Ok(versions)
            }
            .await;

            GolangClient::emit_result(&inner);

            inner
        };

        self.cache
            .versions
            .with_caching(module.to_string(), fut)
            .await
    }
}

use tracing::debug;

use crate::shared::Request;

use super::consts::{BASE_URL_REGISTRY, BASE_URL_SIMPLE, SIMPLE_CONTENT_TYPE};
use super::models::{RegistryMetadata, SimpleMetadata};
use super::util::normalize_name;
use super::{PyPiClient, RequestResult};

impl PyPiClient {
    #[allow(clippy::missing_errors_doc)]
    pub async fn get_simple_metadata(&self, name: &str) -> RequestResult<SimpleMetadata> {
        let normalized = normalize_name(name);
        let simple_url = format!("{BASE_URL_SIMPLE}/{normalized}/");

        let fut = async {
            debug!("Fetching PyPI simple metadata for '{name}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let bytes = Request::get(&simple_url)
                    .with_header("Accept", SIMPLE_CONTENT_TYPE)
                    .send()
                    .await?;
                let text = String::from_utf8(bytes)?;
                Ok(SimpleMetadata::try_from_json(&text)?)
            }
            .await;

            PyPiClient::emit_result(&inner);

            inner
        };

        self.cache
            .simple_metadatas
            .with_caching(simple_url.clone(), fut)
            .await
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn get_registry_metadata(&self, name: &str) -> RequestResult<RegistryMetadata> {
        let normalized = normalize_name(name);
        let registry_url = format!("{BASE_URL_REGISTRY}/{normalized}/json");

        let fut = async {
            debug!("Fetching PyPI registry metadata for '{name}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let bytes = self.request_get(&registry_url).await?;
                let text = String::from_utf8(bytes)?;
                Ok(RegistryMetadata::try_from_json(&text)?)
            }
            .await;

            PyPiClient::emit_result(&inner);

            inner
        };

        self.cache
            .registry_metadatas
            .with_caching(registry_url.clone(), fut)
            .await
    }
}

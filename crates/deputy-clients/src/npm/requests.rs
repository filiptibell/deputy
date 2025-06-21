use tracing::debug;

use super::consts::BASE_URL_REGISTRY;
use super::models::RegistryMetadata;
use super::{NpmClient, RequestResult};

impl NpmClient {
    #[allow(clippy::missing_errors_doc)]
    pub async fn get_registry_metadata(&self, name: &str) -> RequestResult<RegistryMetadata> {
        let name_low = name.to_ascii_lowercase();
        let registry_url = format!("{BASE_URL_REGISTRY}/{name_low}");

        let fut = async {
            debug!("Fetching npm package registry metadatas for '{name}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let bytes = self.request_get(&registry_url).await?;
                let text = String::from_utf8(bytes.clone())?;

                let mut meta = RegistryMetadata::try_from_json(&text)?;
                for (key, value) in &mut meta.versions {
                    value.version.clone_from(key);
                }

                Ok(meta)
            }
            .await;

            NpmClient::emit_result(&inner);

            inner
        };

        self.cache
            .registry_metadatas
            .with_caching(registry_url.clone(), fut)
            .await
    }
}

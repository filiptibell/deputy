use tracing::debug;

use super::consts::*;
use super::models::*;
use super::*;

impl NpmClient {
    pub async fn get_registry_metadata(&self, name: &str) -> RequestResult<RegistryMetadata> {
        let name_low = name.to_ascii_lowercase();
        let registry_url = format!("{BASE_URL_REGISTRY}/{name_low}");

        let fut = async {
            debug!("Fetching npm package registry metadatas for '{name}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let bytes = self.request_get(&registry_url).await?;
                let text = String::from_utf8(bytes.to_vec())?;

                let mut meta = RegistryMetadata::try_from_json(&text)?;
                for (key, value) in meta.versions.iter_mut() {
                    value.version.clone_from(key);
                }

                Ok(meta)
            }
            .await;

            self.emit_result(&inner);

            inner
        };

        self.cache
            .registry_metadatas
            .with_caching(registry_url.clone(), fut)
            .await
    }
}

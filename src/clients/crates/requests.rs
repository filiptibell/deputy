use tracing::debug;

use super::consts::*;
use super::models::*;
use super::*;

impl CratesWrapper {
    pub async fn get_index_metadatas(&self, name: &str) -> RequestResult<Vec<IndexMetadata>> {
        let name_low = name.to_ascii_lowercase();
        let index_url = if name_low.len() <= 2 {
            format!("{BASE_URL_INDEX}/{}/{name_low}", name_low.len())
        } else if name_low.len() == 3 {
            format!("{BASE_URL_INDEX}/3/{}/{name_low}", &name_low[..1])
        } else {
            format!(
                "{BASE_URL_INDEX}/{}/{}/{name_low}",
                &name_low[..2],
                &name_low[2..4]
            )
        };

        let fut = async {
            debug!("Fetching crates index metadatas for '{name}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let mut inner = async {
                let (status, bytes) = self.request(Method::GET, &index_url).await?;
                let text = String::from_utf8(bytes.to_vec())?;
                if !status.is_success() {
                    return Err(ResponseError::from((status, bytes)).into());
                }
                Ok(IndexMetadata::try_from_lines(text.lines().collect())?)
            }
            .await;

            // NOTE: We should sort by most recent version first
            if let Ok(vec) = &mut inner {
                vec.reverse();
            }

            self.emit_result(&inner);

            inner
        };

        self.cache
            .index_metadatas
            .with_caching(index_url.clone(), fut)
            .await
    }

    /**
        Fetches crate data using the crates.io API directly.

        This allows us to access things such as crate description,
        links to repo/docs, as well as download counters, helping
        users of the language server gauge the legitimacy of packages.

        ### Caching

        This method caches its result for the given `name` with a
        duration of *one hour or longer*. For more up-to-date info
        on versions of a package, please use [`get_index_metadatas`].

        ### Rate Limiting

        This method is heavily rate limited, and can only process
        ***one request every five seconds, globally***. This is due
        to the [crates.io crawling policy](https://crates.io/policies).
    */
    pub async fn get_crate_data(&self, name: &str) -> RequestResult<CrateData> {
        let name_low = name.to_ascii_lowercase();
        let crates_url = format!("{BASE_URL_CRATES}/{name_low}{QUERY_STRING_CRATES}");

        let fut = async {
            self.wait_for_crawl_limit().await;
            self.set_crawl_limited();

            debug!("Fetching crate data for '{name}'");

            // NOTE: We make this inner scope so that
            // we can catch and emit all errors at once
            let inner = async {
                let (status, bytes) = self.request(Method::GET, &crates_url).await?;
                if !status.is_success() {
                    return Err(ResponseError::from((status, bytes)).into());
                }
                Ok(serde_json::from_slice::<CrateData>(&bytes)?)
            }
            .await;

            self.emit_result(&inner);

            inner
        };

        self.cache
            .crate_datas
            .with_caching(crates_url.clone(), fut)
            .await
    }
}

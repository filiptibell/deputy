use crate::util::*;

use super::models::*;

#[derive(Debug, Clone)]
pub(super) struct WallyCache {
    pub index_configs: RequestCacheMap<RequestResult<IndexConfig>>,
}

impl WallyCache {
    pub fn new() -> Self {
        Self {
            // NOTE: Registry configs should never really change,
            // so we can use a really long cache here. If a user
            // wants to refresh their config, they can restart
            index_configs: RequestCacheMap::new(
                60 * 24 * 30, // One month
                60 * 24 * 7,  // One week
            ),
        }
    }
}

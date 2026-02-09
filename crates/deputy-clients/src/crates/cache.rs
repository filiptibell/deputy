use crate::shared::{RequestCacheMap, RequestResult};

use super::models::{CrateDataMulti, CrateDataSingle, IndexMetadata, LocalMetadata};

#[derive(Debug, Clone)]
pub(super) struct CratesCache {
    pub index_metadatas: RequestCacheMap<RequestResult<Vec<IndexMetadata>>>,
    pub crate_datas: RequestCacheMap<RequestResult<CrateDataSingle>>,
    pub crate_search: RequestCacheMap<RequestResult<CrateDataMulti>>,
    pub local_metadatas: RequestCacheMap<Option<LocalMetadata>>,
}

impl CratesCache {
    pub fn new() -> Self {
        Self {
            index_metadatas: RequestCacheMap::new_mins(60, 15),
            crate_datas: RequestCacheMap::new_mins(240, 120),
            crate_search: RequestCacheMap::new_mins(480, 240),
            local_metadatas: RequestCacheMap::new_secs(5, 5),
        }
    }
}

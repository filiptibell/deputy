use crate::util::{RequestCacheMap, RequestResult};

use super::models::{CrateDataMulti, CrateDataSingle, IndexMetadata};

#[derive(Debug, Clone)]
pub(super) struct CratesCache {
    pub index_metadatas: RequestCacheMap<RequestResult<Vec<IndexMetadata>>>,
    pub crate_datas: RequestCacheMap<RequestResult<CrateDataSingle>>,
    pub crate_search: RequestCacheMap<RequestResult<CrateDataMulti>>,
}

impl CratesCache {
    pub fn new() -> Self {
        Self {
            index_metadatas: RequestCacheMap::new(60, 15),
            crate_datas: RequestCacheMap::new(240, 120),
            crate_search: RequestCacheMap::new(480, 240),
        }
    }
}

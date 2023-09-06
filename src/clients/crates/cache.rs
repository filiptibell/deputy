use crate::util::*;

use super::models::*;

#[derive(Debug, Clone)]
pub(super) struct CratesCache {
    pub index_metadatas: RequestCacheMap<RequestResult<Vec<IndexMetadata>>>,
    pub crate_datas: RequestCacheMap<RequestResult<CrateData>>,
}

impl CratesCache {
    pub fn new() -> Self {
        Self {
            index_metadatas: RequestCacheMap::new(60, 15),
            crate_datas: RequestCacheMap::new(240, 120),
        }
    }
}

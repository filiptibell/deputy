use crate::util::*;

use super::models::*;
use super::*;

#[derive(Debug, Clone)]
pub(super) struct CratesCache {
    pub index_metadatas: RequestCacheMap<CratesResult<Vec<IndexMetadata>>>,
    pub crate_datas: RequestCacheMap<CratesResult<CrateData>>,
}

impl CratesCache {
    pub fn new() -> Self {
        Self {
            index_metadatas: RequestCacheMap::new(60, 15),
            crate_datas: RequestCacheMap::new(240, 120),
        }
    }
}

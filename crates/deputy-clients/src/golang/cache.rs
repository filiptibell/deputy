use crate::shared::{RequestCacheMap, RequestResult};

use super::models::{Module, Package, SearchResults, Versions};

#[derive(Debug, Clone)]
pub(super) struct GolangCache {
    pub modules: RequestCacheMap<RequestResult<Module>>,
    pub packages: RequestCacheMap<RequestResult<Package>>,
    pub searches: RequestCacheMap<RequestResult<SearchResults>>,
    pub versions: RequestCacheMap<RequestResult<Versions>>,
}

impl GolangCache {
    pub fn new() -> Self {
        Self {
            modules: RequestCacheMap::new_mins(60, 15),
            packages: RequestCacheMap::new_mins(60, 15),
            searches: RequestCacheMap::new_mins(60, 15),
            versions: RequestCacheMap::new_mins(60, 15),
        }
    }
}

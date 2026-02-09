use crate::shared::{RequestCacheMap, RequestResult};

use super::models::ModuleVersion;

#[derive(Debug, Clone)]
pub(super) struct GolangCache {
    pub version_lists: RequestCacheMap<RequestResult<Vec<String>>>,
    pub latest_versions: RequestCacheMap<RequestResult<ModuleVersion>>,
}

impl GolangCache {
    pub fn new() -> Self {
        Self {
            version_lists: RequestCacheMap::new_mins(60, 15),
            latest_versions: RequestCacheMap::new_mins(60, 15),
        }
    }
}

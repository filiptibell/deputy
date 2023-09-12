use crate::util::*;

use super::models::*;

#[derive(Debug, Clone)]
pub(super) struct WallyCache {
    pub package_metadatas: RequestCacheMap<RequestResult<Vec<Metadata>>>,
}

impl WallyCache {
    pub fn new() -> Self {
        Self {
            package_metadatas: RequestCacheMap::new(30, 5),
        }
    }

    pub fn invalidate(&self) {
        self.package_metadatas.invalidate();
    }
}

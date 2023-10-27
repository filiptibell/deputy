use crate::util::*;

use super::models::*;

#[derive(Debug, Clone)]
pub(super) struct NpmCache {
    pub registry_metadatas: RequestCacheMap<RequestResult<RegistryMetadata>>,
}

impl NpmCache {
    pub fn new() -> Self {
        Self {
            registry_metadatas: RequestCacheMap::new(60, 15),
        }
    }
}

use crate::shared::{RequestCacheMap, RequestResult};

use super::models::RegistryMetadata;

#[derive(Debug, Clone)]
pub(super) struct NpmCache {
    pub registry_metadatas: RequestCacheMap<RequestResult<RegistryMetadata>>,
}

impl NpmCache {
    pub fn new() -> Self {
        Self {
            registry_metadatas: RequestCacheMap::new_mins(60, 15),
        }
    }
}

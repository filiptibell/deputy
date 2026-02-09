use crate::shared::{RequestCacheMap, RequestResult};

use super::models::{RegistryMetadata, SimpleMetadata};

#[derive(Debug, Clone)]
pub(super) struct PyPiCache {
    pub simple_metadatas: RequestCacheMap<RequestResult<SimpleMetadata>>,
    pub registry_metadatas: RequestCacheMap<RequestResult<RegistryMetadata>>,
}

impl PyPiCache {
    pub fn new() -> Self {
        Self {
            simple_metadatas: RequestCacheMap::new_mins(60, 15),
            registry_metadatas: RequestCacheMap::new_mins(240, 120),
        }
    }
}

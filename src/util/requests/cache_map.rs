use std::{sync::Arc, time::Duration};

use async_lock::Semaphore;
use dashmap::DashMap;
use futures_lite::Future;
use moka::future::Cache;
use tracing::trace;

type CacheMap<T> = Cache<String, T>;
type Semaphores = Arc<DashMap<String, Semaphore>>;

/**
    Generic cache map for web requests.

    This cache map is both thread-safe and concurrency-safe.
*/
#[derive(Debug, Clone)]
pub struct RequestCacheMap<T: Clone + Send + Sync + 'static> {
    map: CacheMap<T>,
    sems: Semaphores,
}

impl<T: Clone + Send + Sync + 'static> RequestCacheMap<T> {
    /**
        Creates a new cache map.

        - `minutes_to_live` - how many minutes before any
          cached piecec of data gets removed from the cache
        - `minutes_to_idle` - how many minutes **of not being used**
          before a cached piece of data gets removed from the cache
    */
    pub fn new(minutes_to_live: u64, minutes_to_idle: u64) -> Self {
        let map = Cache::builder()
            .max_capacity(64)
            .time_to_live(Duration::from_secs(60 * minutes_to_live))
            .time_to_idle(Duration::from_secs(60 * minutes_to_idle))
            .build();
        RequestCacheMap {
            map,
            sems: Arc::new(DashMap::new()),
        }
    }

    /**
        Invalidates the cache map.

        This will clear out all of the data from the cache and force
        any new requests using `with_caching` to fetch new data.
    */
    pub fn invalidate(&self) {
        self.map.invalidate_all();
    }

    /**
        Run a future with caching and single concurrency limit.

        The given future will run **at most** once at a time, any concurrent calls will
        receive the exact same result, instead being fetched from the cache using the
        provided cache key. There is no guarantee that the future runs, since the result
        may have already been cached, so make sure it does not have any side effects.

        ### Example usage

        ```rust
        let key = "my_cache_key";
        let fut = async move {
            // Do stuff that returns a result
        };

        let cached_result = cache_map.with_caching(key, fut).await // New result
        let cached_cloned = cache_map.with_caching(key, fut).await // Same result
        let cached_cloned2 = cache_map.with_caching(key, fut).await // Same result

        // The `fut` future will only run once (!!!)
        let cached_result = select! {
            v = cache_map.with_caching(key, fut) => v,
            v = cache_map.with_caching(key, fut) => v,
            v = cache_map.with_caching(key, fut) => v,
        }

        cache_map.invalidate()

        let cached_fresh = cache_map.with_caching(key, fut).await // New result
        ```
    */
    pub async fn with_caching<F>(&self, key: impl Into<String>, f: F) -> T
    where
        F: Future<Output = T>,
    {
        let key = key.into();

        // Return cached value right away if possible
        if let Some(cached) = self.map.get(&key) {
            trace!("Cache hit (1): {key}");
            return cached.clone();
        }

        // Wait for permission to try to perform the request -
        // guarantees at most one requester at a time per key
        let sem = self
            .sems
            .entry(key.clone())
            .or_insert_with(|| Semaphore::new(1));
        let _guard = sem.acquire().await;

        // We have permission, but the cache may have been updated, check again
        if let Some(cached) = self.map.get(&key) {
            trace!("Cache hit (2): {key}");
            return cached.clone();
        }

        // Not cached, and we have permission, so perform the request
        trace!("Performing cached request: {key}");
        let result = f.await;
        self.map.insert(key.clone(), result.clone()).await;
        result
    }
}

use std::{sync::Arc, time::Duration};

use async_broadcast::{broadcast, Sender};
use dashmap::DashMap;
use futures::Future;
use moka::future::Cache;
use smol::Timer;
use tracing::trace;

type CacheMap<T> = Cache<String, T>;

// Map of senders, used to notify any listeners
// that are waiting for a request to finish and
// a cache value to become available to clone
type Senders<T> = Arc<DashMap<String, Sender<T>>>;

/**
    Generic cache map for web requests.

    This cache map is both thread-safe and concurrency-safe.
*/
#[derive(Debug, Clone)]
pub struct RequestCacheMap<T: Clone + Send + Sync + 'static> {
    map: CacheMap<T>,
    sends: Senders<T>,
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
            sends: Arc::new(DashMap::new()),
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

        cache_map.invalidate()

        let cached_fresh = cache_map.with_caching(key, fut).await // New result
        ```
    */
    pub async fn with_caching<F>(&self, key: impl Into<String>, f: F) -> T
    where
        F: Future<Output = T>,
    {
        let key = key.into();

        let sends = Arc::clone(&self.sends);
        let send = sends.get(&key).map(|r| r.clone());

        if let Some(send) = send {
            if let Ok(res) = send.new_receiver().recv().await {
                return res;
            } else {
                // Existing request was cancelled / dropped, try again
            }
        }

        match self.map.get(&key) {
            Some(cached) => cached.clone(),
            None => {
                // HACK: Spawn a timeout task that will clear out any
                // senders if for some reason this request was cancelled
                // We should really do this on future being dropped instead
                let sends_key = key.clone();
                let sends_timeout = Arc::clone(&sends);
                smol::spawn(async move {
                    Timer::after(Duration::from_secs(5)).await;
                    if sends_timeout.remove(&sends_key).is_some() {
                        trace!("Request was cancelled, cleaning up")
                    }
                })
                .detach();

                let (send, _) = broadcast(1);
                sends.insert(key.clone(), send.clone());

                let result = f.await;

                self.map.insert(key.clone(), result.clone()).await;

                sends.remove(&key);
                send.try_broadcast(result.clone()).ok();

                result
            }
        }
    }
}

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::Future;
use moka::future::Cache;
use tokio::sync::broadcast::{self, Sender};

type CacheMap<T> = Cache<String, T>;

// Map of senders, used to notify any listeners
// that are waiting for a request to finish and
// a cache value to become available to clone
type Senders<T> = Arc<Mutex<HashMap<String, Sender<T>>>>;

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
            sends: Arc::new(Mutex::new(HashMap::new())),
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
        let send = { sends.lock().unwrap().get(&key).cloned() };

        if let Some(send) = send {
            return send
                .subscribe()
                .recv()
                .await
                .expect("Unexpectedly dropped sender");
        }

        match self.map.get(&key) {
            Some(cached) => cached.clone(),
            None => {
                let send = {
                    let (send, _) = broadcast::channel(1);
                    sends.lock().unwrap().insert(key.clone(), send.clone());
                    send
                };

                let result = f.await;

                self.map.insert(key.clone(), result.clone()).await;

                {
                    sends.lock().unwrap().remove(&key);
                }

                send.send(result.clone()).ok();

                result
            }
        }
    }
}

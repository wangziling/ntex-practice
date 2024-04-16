use moka::future::Cache;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub type MemoryCacheKey = &'static str;
pub type MemoryCacheValue = serde_json::Value;

pub type MemoryCacheGlobal = Arc<RwLock<MemoryCache>>;

pub struct MemoryCache {
    client: Cache<MemoryCacheKey, MemoryCacheValue>,
}

impl Deref for MemoryCache {
    type Target = Cache<MemoryCacheKey, MemoryCacheValue>;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for MemoryCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

pub struct MemoryCacheExtension(MemoryCacheGlobal);

impl Deref for MemoryCacheExtension {
    type Target = MemoryCacheGlobal;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn generate() -> MemoryCacheGlobal {
    debug!("Generating the Memory cache.");

    Arc::new(RwLock::new(MemoryCache {
        client: Cache::builder()
            // Time to live (TTL): 30 minutes
            .time_to_live(Duration::from_secs(30 * 60))
            // Time to idle (TTI):  5 minutes
            .time_to_idle(Duration::from_secs(5 * 60))
            // This cache will hold up to 32MiB of values.
            .max_capacity(32 * 1024 * 1024)
            .build(),
    }))
}

#[macro_export]
macro_rules! memory_cache_make_sure {
    ($cache: expr, $task: expr) => {
        $task;

        // This make sure what we changed in `$task` will be effective "immediately".
        $cache.run_pending_tasks().await;
    };
}

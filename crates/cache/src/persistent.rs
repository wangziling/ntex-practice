use moka::future::Cache;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub type PersistentCacheKey = &'static str;
pub type PersistentCacheValue = serde_json::Value;

pub type PersistentCacheGlobal = Arc<RwLock<PersistentCache>>;

pub struct PersistentCache {
    client: Cache<PersistentCacheKey, PersistentCacheValue>,
}

impl Deref for PersistentCache {
    type Target = Cache<PersistentCacheKey, PersistentCacheValue>;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for PersistentCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

pub struct PersistentCacheExtension(PersistentCacheGlobal);

impl Deref for PersistentCacheExtension {
    type Target = PersistentCacheGlobal;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn generate() -> PersistentCacheGlobal {
    debug!("Generating the persistent cache.");

    Arc::new(RwLock::new(PersistentCache {
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
macro_rules! persistent_mark_sure {
    ($cache: expr, $task: expr) => {
        $task;

        // This make sure what we changed in `$task` will be effective "immediately".
        $cache.run_pending_tasks().await;
    };
}

#[macro_use]
extern crate tracing;

mod distribute;
mod persistent;

use once_cell::sync::Lazy;
use web_core::prelude::*;

#[allow(unused)]
pub mod prelude {
    pub use crate::distribute::prelude::*;
    pub use crate::distribute::{
        DistributeCache, DistributeCacheConfig, DistributeCacheExtension, DistributeCacheGlobal, DistributeCacheKey,
    };
    pub use crate::persistent::{
        PersistentCache, PersistentCacheExtension, PersistentCacheGlobal, PersistentCacheKey, PersistentCacheValue,
    };
    pub use crate::persistent_mark_sure;
}

/// Distribute cache can only be accessed in `app_state`.
pub async fn generate_distribute_cache(
    config: distribute::DistributeCacheConfig,
) -> Result<distribute::DistributeCacheGlobal> {
    debug!("Connecting to the distribute cache.");

    distribute::generate(config).await
}

/// Persistent cache should be a global state.
pub static PERSISTENT_CACHE: Lazy<persistent::PersistentCacheGlobal> = Lazy::new(persistent::generate);

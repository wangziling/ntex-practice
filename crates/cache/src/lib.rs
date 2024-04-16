#[macro_use]
extern crate tracing;

mod distribute;
mod memory;

use once_cell::sync::Lazy;
use web_core::prelude::*;

#[allow(unused)]
pub mod prelude {
    pub use crate::distribute::prelude::*;
    pub use crate::distribute::{
        DistributeCache, DistributeCacheConfig, DistributeCacheExtension, DistributeCacheGlobal, DistributeCacheKey,
    };
    pub use crate::memory::{MemoryCache, MemoryCacheExtension, MemoryCacheGlobal, MemoryCacheKey, MemoryCacheValue};
    pub use crate::memory_cache_make_sure;
}

/// Distribute cache can only be accessed in `app_state`.
pub async fn generate_distribute_cache(
    config: distribute::DistributeCacheConfig,
) -> Result<distribute::DistributeCacheGlobal> {
    debug!("Connecting to the distribute cache.");

    distribute::generate(config).await
}

/// Memory cache should be a global state.
pub static MEMORY_CACHE: Lazy<memory::MemoryCacheGlobal> = Lazy::new(memory::generate);

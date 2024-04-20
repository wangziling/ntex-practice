#[macro_use]
extern crate tracing;

mod error;
mod impls;

use once_cell::sync::Lazy;
use web_core::prelude::*;

pub mod prelude {
    pub use crate::impls::distribute::prelude::*;
    pub use crate::impls::distribute::{
        DistributeCache, DistributeCacheConfig, DistributeCacheExtension, DistributeCacheGlobal, DistributeCacheKey,
    };

    pub use crate::impls::memory::prelude::*;
    pub use crate::impls::memory::{
        MemoryCache, MemoryCacheExtension, MemoryCacheGlobal, MemoryCacheKey, MemoryCacheValue,
    };
    pub use crate::memory_cache_make_sure;
}

/// Distribute cache can only be accessed in `app_state`.
pub async fn generate_distribute_cache(
    config: crate::impls::distribute::DistributeCacheConfig,
) -> Result<crate::impls::distribute::DistributeCacheGlobal> {
    debug!("Connecting to the distribute cache.");

    impls::distribute::generate(config).await
}

/// Memory cache should be a global state.
pub static MEMORY_CACHE: Lazy<crate::impls::memory::MemoryCacheGlobal> = Lazy::new(impls::memory::generate);

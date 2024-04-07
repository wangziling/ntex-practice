#[cfg(feature = "enable-redis")]
use web_core::prelude::*;

#[cfg(feature = "enable-redis")]
mod distribute;

#[allow(unused)]
pub mod prelude {
    #[cfg(feature = "enable-redis")]
    pub use crate::cache::distribute::{
        prelude::*, DistributeCache, DistributeCacheConfig, DistributeCacheExtension, DistributeCacheGlobal,
        DistributeCacheKey,
    };
}

#[cfg(feature = "enable-redis")]
/// Distribute cache can only be accessed in `app_state`.
pub async fn generate_distribute_cache(
    config: distribute::DistributeCacheConfig,
) -> Result<distribute::DistributeCacheGlobal> {
    debug!("Connecting to the distribute cache.");

    distribute::generate(config).await
}

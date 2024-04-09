use web_core::prelude::*;

mod distribute;

pub mod prelude {
    pub use web_cache::distribute::{
        prelude::*, DistributeCache, DistributeCacheConfig, DistributeCacheExtension, DistributeCacheGlobal,
        DistributeCacheKey,
    };
}

/// Distribute cache can only be accessed in `app_state`.
pub async fn generate_distribute_cache(
    config: distribute::DistributeCacheConfig,
) -> Result<distribute::DistributeCacheGlobal> {
    debug!("Connecting to the distribute cache.");

    distribute::generate(config).await
}

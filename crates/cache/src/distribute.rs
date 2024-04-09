#![allow(unused)]
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Duration,
};

use fred::prelude::*;
use web_core::prelude::*;

pub type DistributeCacheKey = &'static str;

pub type DistributeCacheGlobal = Arc<DistributeCache>;
pub type DistributeCacheConfig = RedisConfig;

pub mod prelude {
    pub use fred::prelude::*;
}

pub struct DistributeCache {
    client: RedisClient,
}

impl Deref for DistributeCache {
    type Target = RedisClient;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for DistributeCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

pub struct DistributeCacheExtension(DistributeCacheGlobal);

impl Deref for DistributeCacheExtension {
    type Target = DistributeCacheGlobal;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn generate(config: DistributeCacheConfig) -> Result<DistributeCacheGlobal> {
    let client = DistributeCache {
        client: RedisClient::new(
            config,
            Some(PerformanceConfig {
                // 10s.
                default_command_timeout: Duration::from_secs(10),
                ..Default::default()
            }),
            None,
            Some(ReconnectPolicy::new_constant(u32::MAX, 8_000)),
        ),
    };

    debug!("Connecting to the redis cache.");

    // No need to wait for being connected.
    #[allow(clippy::let_underscore_future)]
    let _ = client.connect();

    // No need to use the `?` to wait for being connected.
    let _ = client.wait_for_connect().await;

    Ok(Arc::new(client))
}
